#[macro_use]
mod macros;

use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

use jsrs_common::types::coerce::{AsBool,AsNumber};

use french_press::ScopeManager;
use french_press::alloc::AllocBox;
use jsrs_parser::lalr::parse_Stmt;
use jsrs_common::ast::*;
use jsrs_common::ast::Exp::*;
use jsrs_common::ast::Stmt::*;

use jsrs_common::types::binding::Binding;
use jsrs_common::types::js_fn::JsFnStruct;
use jsrs_common::types::js_obj::JsObjStruct;
use jsrs_common::types::js_str::JsStrStruct;
use jsrs_common::types::js_var::{JsVar, JsType, JsPtrEnum, JsKey, JsPtrTag};
use jsrs_common::types::js_var::JsType::*;
use jsrs_common::backend::Backend;
use number::eval_binop;
use var::*;
use error::{Result, JsError};
use unescape::unescape;


/// Evaluate a string containing some JavaScript statements (or sequences of statements).
/// Returns a JsVar which is the return value of those statements.
pub fn eval_string(string: &str, state: Rc<RefCell<ScopeManager>>) -> Result<JsVarValue> {
    match parse_Stmt(string) {
        Ok(stmt) => { Ok(eval_stmt(&stmt, state).expect("Error evaluating statement").0) }
        Err(e) => Err(JsError::ParseError(format!("{:?}", e))),
    }
}

/// Evaluate a single JS statement (which may be a block or sequence of statements).
/// Returns tuple of (evaluated final value, return value), where return value requires that
/// `return` be used to generate it.
pub fn eval_stmt(s: &Stmt, state: Rc<RefCell<ScopeManager>>) -> Result<(JsVarValue, JsReturnValue)> {
    match *s {
        // var_string = exp;
        Assign(ref var_string, ref exp) => {
            let (new_var, js_ptr) = eval_exp(exp, state.clone());
            // TODO error handling
            let (mut js_var, _) = state.deref().borrow().load(&Binding::new(var_string.clone())).unwrap();
            js_var.t = new_var.t;

            let old_binding = js_var.unique.clone();
            let _ = js_var.deanonymize(var_string);
            let _ = state.deref().borrow_mut().rename_closure(&old_binding, &js_var.unique);

            // Clone the js_var to store into the ScopeManager
            let cloned = js_var.clone();

            match state.deref().borrow_mut().store(cloned, js_ptr.clone()) {
                Ok(_) => (),
                e @ Err(_) => println!("{:?}", e),
            }

            Ok(((js_var, js_ptr), None))
        },

        // exp;
        BareExp(ref exp) => Ok((eval_exp(exp, state.clone()), None)),

        // var var_string = exp
        Decl(ref var_string, ref exp) => {
            let (mut js_var, js_ptr) = eval_exp(exp, state.clone());
            let old_binding = js_var.unique.clone();
            let _ = js_var.deanonymize(var_string);

            let _ = state.deref().borrow_mut().rename_closure(&old_binding, &js_var.unique);

            match state.deref().borrow_mut().alloc(js_var, js_ptr) {
                Ok(_) => Ok((scalar(JsUndef), None)),
                Err(e) => {
                    Err(JsError::GcError(e))
                }
            }
        },

        // if (condition) { if_block } else { else_block }
        If(ref condition, ref if_block, ref else_block) => {
            // evaluate expression
            if eval_exp(&condition, state.clone()).0.as_bool() {
                // condition = true, evaluate if-block.
                eval_stmt(&*if_block, state.clone())
            } else {
                // condition = false
                // evaluate else-block if it exists, otherwise return undefined.
                if let Some(ref block) = *else_block {
                    eval_stmt(&*block, state.clone())
                } else {
                    Ok((scalar(JsUndef), None))
                }
            }
        },

        Empty => Ok((scalar(JsUndef), None)),

        // return exp
        Ret(ref exp) => {
            let js_var = eval_exp(&exp, state.clone());
            Ok((js_var.clone(), Some(js_var)))
        }

        // a sequence of any two expressions
        Seq(ref s1, ref s2) => {
            try!(eval_stmt(&*s1, state.clone()));
            eval_stmt(&*s2, state.clone())
        },

        // throw <expression>;
        Throw(..) => unimplemented!(),

        // try { block } [catch <expression> { block} &&/|| finally { block }]
        Try(..) => unimplemented!(),

        // while (condition) { block }
        While(ref condition, ref block) => {
            let mut ret_val = None;
            loop {
                if eval_exp(&condition, state.clone()).0.as_bool() {
                    // TODO: check to see if a return stmt has been reached.
                    let (_, v) = eval_stmt(&*block, state.clone()).unwrap();
                    ret_val = v;
                } else {
                    // condition is no longer true, return a return value
                    return Ok((scalar(JsUndef), ret_val));
                }
            }
        }
    }
}

/// Evaluate an expression into a JsVar.
pub fn eval_exp(e: &Exp, state: Rc<RefCell<ScopeManager>>) -> JsVarValue {
    match e {
        // e1 [op] e2
        &BinExp(ref e1, ref op, ref e2) => {
            let val1 = eval_exp(e1, state.clone()).0;
            let val2 = eval_exp(e2, state.clone()).0;

            let result = eval_binop(op, val1, val2);
            scalar(result)
        }
        &Bool(b) => scalar(JsBool(b)),

        // fun_name([arg_exp1, arg_exps])
        &Call(ref fun_name, ref arg_exps) => {
            let (fun_binding, fun_ptr) = eval_exp(fun_name, state.clone());

            // Create vector of arguments, evaluated to JsVars.
            let mut args = Vec::new();
            for exp in arg_exps {
                args.push(eval_exp(exp, state.clone()));
            }

            let js_fn_struct = match fun_ptr {
                Some(JsPtrEnum::JsFn(fun)) => fun,
                Some(JsPtrEnum::NativeFn(func)) => return func.call(state.clone(), None, args),
                Some(_) => panic!("ReferenceError: {:?} is not a function", fun_name),
                None => match state.deref().borrow().load(&fun_binding.binding) {
                    Ok((_, Some(JsPtrEnum::JsFn(fun)))) => fun,
                    Ok(_) => panic!("ReferenceError: {:?} is not a function", fun_name),
                    Err(_) => panic!("ReferenceError: {:?} is not defined", fun_name),
                }
            };

            match js_fn_struct.name {
                Some(_) => state.deref().borrow_mut().push_scope(e),
                None => state.deref().borrow_mut().push_closure_scope(&fun_binding.unique).expect("Unable to push closure scope")
            };

            for param in js_fn_struct.params {
                let mut arg = if args.is_empty() {
                    scalar(JsUndef)
                } else {
                    args.remove(0)
                };

                arg.0.binding = Binding::new(param.to_owned());
                state.deref().borrow_mut().alloc(arg.0, arg.1)
                .expect("Unable to store function argument in scope");
            }

            let (_, v) = eval_stmt(&js_fn_struct.stmt, state.clone()).expect("Error running function body");

            // If the return value of a function is `None` (void),
            // or is not a pointer to a function, a closure is not being
            // returned from the function. If the function is returning a
            // function, and the function being returned has no name, a closure
            // is being returned.
            let returning_closure = v.as_ref().map_or(None, |ref var| {
                match var.0.t {
                    JsType::JsPtr(ref tag) => match tag {
                        &JsPtrTag::JsFn {..} => Some(var.0.unique.clone()),
                        _ => None,
                    },
                    _ => None,
                }
            });

            // Should we yield here? Not sure, so for now it doesn't
            state.deref().borrow_mut().pop_scope(returning_closure, false).expect("Unable to clear scope for function");

            v.unwrap_or(scalar(JsUndef))
        }

        // function([param1, params]) { body }
        // function opt_binding([param1, params]) { body }
        &Defun(ref opt_binding, ref params, ref body) => {
            let js_fun = JsFnStruct::new(opt_binding, params, &**body);

            let var = if let &Some(ref s) = opt_binding {
                JsVar::bind(s, JsPtr(JsPtrTag::JsFn { name: opt_binding.clone() }))
            } else {
                JsVar::new(JsPtr(JsPtrTag::JsFn { name: None }))
            };

            if let Err(_) = state.deref().borrow_mut().alloc(var.clone(), Some(JsPtrEnum::JsFn(js_fun.clone()))) {
                panic!("error storing function into state");
            }

            (var, Some(JsPtrEnum::JsFn(js_fun)))
        },

        // var.binding
        &InstanceVar(ref instance_exp, ref var) => {
            // TODO: this needs better type-reasoning and errors
            let (instance_var, var_ptr) = eval_exp(instance_exp, state.clone());
            if let JsPtr(_) = instance_var.t {
                match var_ptr {
                    Some(JsPtrEnum::JsObj(obj_struct)) => {
                        let try_inner = obj_struct.dict.get(&JsKey::JsStr(JsStrStruct::new(var)));
                        if let Some(inner_var) = try_inner {
                            return (inner_var.clone(), None);
                        } else {
                            return scalar(JsUndef);
                        }
                    },
                    // TODO: all JsPtrs can have instance vars/methods, not just JsObjs
                    _ => unimplemented!()
                }
            } else {
                // TODO: Things which are not ptrs can also have instance vars/methods
                unimplemented!()
            }
        },

        &Null => {
            scalar(JsNull)
        },

        &Float(f) => scalar(JsType::JsNum(f)),
        &Neg(ref exp) => scalar(JsNum(-eval_exp(exp, state.clone()).0.as_number())),
        &Pos(ref exp) => scalar(JsNum(eval_exp(exp, state.clone()).0.as_number())),

        &PostDec(ref exp) => (eval_float_post_op!(exp, f, f - 1.0, state), None),
        &PostInc(ref exp) => (eval_float_post_op!(exp, f, f + 1.0, state), None),
        &PreDec(ref exp)  => (eval_float_pre_op!(exp, f, f - 1.0, state),  None),
        &PreInc(ref exp)  => (eval_float_pre_op!(exp, f, f + 1.0, state),  None),

        &NewObject(_, _) => unimplemented!(),
        &Object(ref fields) => {
            let mut kv_tuples = Vec::new();
            for f in fields {
                let f_key = JsKey::JsStr(JsStrStruct::new(&f.0));
                // TODO: handle obj as key/value pair
                let f_exp = eval_exp(&*f.1, state.clone()).0;
                kv_tuples.push((f_key, f_exp, None));
            }
            let obj = JsObjStruct::new(None, "", kv_tuples, &mut AllocBox::new());
            (JsVar::new(JsPtr(JsPtrTag::JsObj)), Some(JsPtrEnum::JsObj(obj)))
        },

        &Str(ref s) => {
            let var = JsVar::new(JsPtr(JsPtrTag::JsStr));
            let ptr = Some(JsPtrEnum::JsStr(JsStrStruct::new(&unescape(s).expect("Invalid string"))));
            (var, ptr)
        }
        &TypeOf(ref e) => (JsVar::new(JsPtr(JsPtrTag::JsStr)), Some(JsPtrEnum::JsStr(JsStrStruct::new(&eval_exp(e, state.clone()).0.type_of())))),
        &Undefined => scalar(JsUndef),
        &Var(ref var_binding) => {
            state.deref().borrow().load(&Binding::new(var_binding.clone()))
                .expect("ReferenceError: {} is not defined")
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;
    use french_press::init_gc;
    use jsrs_common::types::js_var::JsType;

    #[test]
    fn test_eval_literals() {
        let state = Rc::new(RefCell::new(init_gc()));
        assert_eq!(JsType::JsNum(5.0f64), eval_string("5.0;\n", state.clone()).unwrap().0.t);
        assert_eq!(JsType::JsNum(0.0f64), eval_string("0.0;\n", state.clone()).unwrap().0.t);
        assert_eq!(JsType::JsUndef, eval_string("undefined;\n", state.clone()).unwrap().0.t);
    }

    //// TODO: handle `var` and no `var` separately
    //#[test]
    //fn test_store_state() {
    //    let mut state = HashMap::new();
    //    assert_eq!(JsUndefined, eval_string("var a = 1;\n", &mut state));
    //    assert_eq!(JsNumber(2.0f64), eval_string("a = 2;\n", &mut state));
    //    assert_eq!(JsUndefined, eval_string("var b = 3;\n", &mut state));
    //    assert_eq!(JsNumber(4.0f64), eval_string("c = 4;\n", &mut state));
    //}

    #[test]
    fn test_inc_dec() {
        // let state = Rc::new(RefCell::new(init_gc()));
        //assert_eq!(JsType::JsNum(1.0f64), eval_string("var a = 1;\n", &mut state).t);
        //assert_eq!(&JsType::JsNum(1.0), state.load(&Binding::new("a")).unwrap());

        //assert_eq!(JsType::JsNum(1.0f64), eval_string("a++;\n", &mut state));
        //assert_eq!(&JsNumber(2.0f64), state.get("a").unwrap());

        //assert_eq!(JsNumber(3.0f64), eval_string("++a;\n", &mut state));
        //assert_eq!(&JsNumber(3.0f64), state.get("a").unwrap());

        //assert_eq!(JsNumber(3.0f64), eval_string("a--;\n", &mut state));
        //assert_eq!(&JsNumber(2.0f64), state.get("a").unwrap());

        //assert_eq!(JsNumber(1.0f64), eval_string("--a;\n", &mut state));
        //assert_eq!(&JsNumber(1.0f64), state.get("a").unwrap());
    }

    #[test]
    fn test_binexp() {
        let state = Rc::new(RefCell::new(init_gc()));
        assert_eq!(JsType::JsNum(6.0f64),  eval_string("2.0 + 4.0;\n", state.clone()).unwrap().0.t);
        assert_eq!(JsType::JsNum(0.5f64),  eval_string("2.0 / 4.0;\n", state.clone()).unwrap().0.t);
        assert_eq!(JsType::JsNum(-2.0f64), eval_string("2.0 - 4.0;\n", state.clone()).unwrap().0.t);
        assert_eq!(JsType::JsNum(8.0f64),  eval_string("2.0 * 4.0;\n", state.clone()).unwrap().0.t);
    }
}
