#[macro_use]
mod macros;

use std::cell::RefCell;
use std::rc::Rc;

use native::get_array_proto;
use number::eval_binop;
use var::*;

use french_press::ScopeManager;
use jsrs_parser::lalr::parse_Stmt;
use jsrs_common::gc_error::GcError;
use jsrs_common::js_error::{self, JsError};
use jsrs_common::ast::*;
use jsrs_common::ast::Exp::*;
use jsrs_common::ast::Stmt::*;
use jsrs_common::types::coerce::{AsBool, AsNumber, AsString};
use jsrs_common::types::binding::Binding;
use jsrs_common::types::js_fn::JsFnStruct;
use jsrs_common::types::js_obj::JsObjStruct;
use jsrs_common::types::js_str::JsStrStruct;
use jsrs_common::types::js_var::{JsVar, JsType, JsPtrEnum, JsKey, JsPtrTag};
use jsrs_common::types::js_var::JsType::*;
use jsrs_common::backend::Backend;

use unescape::unescape;


/// Evaluate a string containing some JavaScript statements (or sequences of statements).
/// Returns a JsVar which is the return value of those statements.
pub fn eval_string(string: &str, state: Rc<RefCell<ScopeManager>>) -> js_error::Result<JsVarValue> {
    match parse_Stmt(string) {
        Ok(stmt) => {
            Ok(try!(eval_stmt(&stmt, state)).0)
        }
        Err(e) => Err(JsError::ParseError(format!("{:?}", e))),
    }
}

pub fn eval_stmt_block(block: &Vec<Stmt>, state: Rc<RefCell<ScopeManager>>)
        -> js_error::Result<(JsVarValue, JsReturnValue)> {
    let mut ret = (scalar(JsUndef), None);
    for stmt in &*block {
        ret = try!(eval_stmt(stmt, state.clone()));
        if let Some(..) = ret.1 {
            return Ok(ret);
        }
    }
    Ok(ret)
}

/// Evaluate a single JS statement (which may be a block or sequence of statements).
/// Returns tuple of (evaluated final value, return value), where return value requires that
/// `return` be used to generate it.
pub fn eval_stmt(s: &Stmt, state: Rc<RefCell<ScopeManager>>)
        -> js_error::Result<(JsVarValue, JsReturnValue)> {
    match *s {
        // var_string = exp;
        Assign(ref lhs, ref exp) => {
            let (rhs_var, rhs_ptr) = try!(eval_exp(exp, state.clone()));

            let var = match lhs {
                &Var(ref string) => {
                    let result = state.borrow_mut().load(&Binding::new(string.to_owned()));
                    let (mut var, ptr) = match result {
                        Ok((v, p)) => (v, p),
                        Err(GcError::Load(_)) => return eval_stmt(&Decl(string.to_owned(), exp.clone()), state.clone()),
                        Err(e) => return Err(JsError::from(e)),
                    };

                    match ptr {
                        Some(JsPtrEnum::NativeVar(mut native_var)) => {
                            native_var.set(state.clone(), None, rhs_var, rhs_ptr);
                            return Ok(((native_var.var, native_var.ptr.map(|x| *x)), None));
                        }
                        _ => ()
                    };

                    var.t = rhs_var.t.clone();
                    let old_binding = var.unique.clone();
                    var.mangle(string);
                    let _ = state.borrow_mut().rename_closure(&old_binding, &var.unique);
                    try!(state.borrow_mut().store(rhs_var.clone(), rhs_ptr.clone()));
                    var
                }
                &InstanceVar(ref e, ref string) => {
                    let (var, ptr) = try!(eval_exp(&e.clone(), state.clone()));

                    let mut obj = match ptr.clone() {
                        Some(JsPtrEnum::JsObj(obj)) => obj,
                        _ => return Ok(((rhs_var, rhs_ptr), None))
                    };


                    let native_var = match obj.dict.get(&js_str_key(string)) {
                        Some(ref var) => {
                            let state_ref = state.borrow_mut();
                            let alloc_box = state_ref.alloc_box.borrow_mut();
                            alloc_box.find_id(&var.unique).map(|p| p.borrow().clone())
                        }
                        None => None
                    };

                    if let Some(JsPtrEnum::NativeVar(mut nv)) = native_var {
                        nv.set(state.clone(), ptr.clone()
                               .map(|x| (var.clone(), x)), rhs_var, rhs_ptr);
                        return Ok(((nv.var.clone(), nv.clone().ptr.map(|x| *x)), None));
                    }

                    let mut state_ref = state.borrow_mut();
                    obj.add_key(&var.unique,
                                JsKey::JsStr(JsStrStruct::new(string)),
                                rhs_var.clone(), rhs_ptr.clone(),
                                &mut *(state_ref.alloc_box.borrow_mut()));
                    rhs_var
                }
                _ => return Err(JsError::invalid_lhs())
            };

            Ok(((var, rhs_ptr), None))
        },

        // exp;
        BareExp(ref exp) => Ok((try!(eval_exp(exp, state.clone())), None)),
        Break => Err(JsError::SyntaxError(String::from("Illegal break statement"))),
        Continue => Err(JsError::SyntaxError(String::from("Illegal continue statement"))),

        // var var_string = exp
        Decl(ref var_string, ref exp) => {
            let (mut js_var, js_ptr) = try!(eval_exp(exp, state.clone()));
            let old_binding = js_var.unique.clone();
            js_var.binding = Binding::new(var_string.clone());

            let _ = state.borrow_mut().rename_closure(&old_binding, &js_var.unique);

            match state.borrow_mut().alloc(js_var, js_ptr) {
                Ok(_) => Ok((scalar(JsUndef), None)),
                Err(e) => {
                    Err(JsError::GcError(e))
                }
            }
        },

        // if (condition) { if_block } else { else_block }
        If(ref condition, ref if_block, ref else_block) => {
            // evaluate expression
            if try!(eval_exp(&condition, state.clone())).0.as_bool() {
                // condition = true => evaluate if-block.
                return eval_stmt_block(&*if_block, state.clone());
            } else {
                // condition = false =>
                // evaluate else-block if it exists, otherwise return undefined.
                return eval_stmt_block(&*else_block, state.clone());
            }
        },

        Empty => Ok((scalar(JsUndef), None)),

        // return exp
        Ret(ref exp) => {
            let js_var = try!(eval_exp(&exp, state.clone()));
            Ok((js_var.clone(), Some(js_var)))
        }

        // a sequence of any two expressions
        Seq(ref s1, ref s2) => {
            try!(eval_stmt(&*s1, state.clone()));
            eval_stmt(&*s2, state.clone())
        },

        // throw <expression>;
        Throw(ref exp) => {
            let (var, ptr) = try!(eval_exp(exp, state));
            Err(JsError::JsVar((var, ptr)))
        }

        // try { block } [catch <expression> { block} &&/|| finally { block }]
        Try(ref try_block, ref catch_var, ref catch_block, ref finally_block) => {
            match eval_stmt_block(try_block, state.clone()) {
                Ok(_) => {
                    try!(eval_stmt_block(finally_block, state.clone()));
                    return Ok((scalar(JsUndef), None));
                }
                Err(_) => {
                    // TODO: figure out how to bind error to variable in scope.
                    let mut var = JsVar::new(JsType::JsPtr(JsPtrTag::JsObj));
                    var.binding = Binding::new(catch_var.clone());
                    {
                        let mut state_ref = state.borrow_mut();
                        let obj = JsObjStruct::new(None, "", Vec::new(),
                            &mut *(state_ref.alloc_box.borrow_mut()));
                        // Add error to scope.
                        state_ref.push_scope(&Exp::Null);
                        try!(state_ref.alloc(var, Some(JsPtrEnum::JsObj(obj))));
                    }

                    try!(eval_stmt_block(catch_block, state.clone()));

                    try!(state.borrow_mut().pop_scope(None, false));

                    try!(eval_stmt_block(finally_block, state.clone()));
                    return Ok((scalar(JsUndef), None));
                }
            }
        }

        // while (condition) { block }
        VarDecl(ref s) => {
            let (mut var, ptr) = scalar(JsUndef);
            var.binding = Binding::new(s.clone());
            match state.borrow_mut().alloc(var, ptr) {
                Ok(_) => Ok((scalar(JsUndef), None)),
                Err(e) => {
                    Err(JsError::GcError(e))
                }
            }
        }

        // while (condition) { block }
        While(ref condition, ref block) => {
            loop {
                if try!(eval_exp(&condition, state.clone())).0.as_bool() {
                    let (js_var, js_ret_var) = try!(eval_stmt_block(block, state.clone()));
                    if let Some(..) = js_ret_var {
                        return Ok((js_var, js_ret_var));
                    }
                } else {
                    // condition is no longer true, return a return value
                    return Ok((scalar(JsUndef), None));
                }
            }
        }
    }
}

/// Evaluate an expression into a JsVar.
pub fn eval_exp(e: &Exp, state: Rc<RefCell<ScopeManager>>) -> js_error::Result<JsVarValue> {
    match e {
        // [ e1, e2, ... ]
        &Array(ref elems) => {
            let proto = Some(Box::new(get_array_proto(elems.len() as f64, state.clone())));

            let mut kv_tuples = Vec::new();
            for (i, elem) in elems.iter().enumerate() {
                let i_str = format!("{}", i);
                let (mut var, ptr) = try!(eval_exp(elem, state.clone()));
                let key = js_str_key(&i_str);
                var.mangle(&i_str);
                kv_tuples.push((key, var, ptr));
            }

            let mut state_ref = state.borrow_mut();
            let obj = JsObjStruct::new(proto, "array", kv_tuples,
                                       &mut *(state_ref.alloc_box.borrow_mut()));

            Ok((JsVar::new(JsType::JsPtr(JsPtrTag::JsObj)), Some(JsPtrEnum::JsObj(obj))))
        }
        &BitNot(ref exp) => {
            let i = try!(eval_exp(exp, state.clone())).0.as_number() as i32;
            Ok(scalar(JsNum((!i) as f64)))
        }
        // e1 [op] e2
        &BinExp(ref e1, ref op, ref e2) => {
            let result = try!(eval_binop(op, e1, e2, state.clone()));
            Ok(scalar(result))
        }
        &Bool(b) => Ok(scalar(JsBool(b))),

        // fun_name([arg_exp1, arg_exps])
        &Call(ref fun_name, ref arg_exps) => {
            let ((fun_binding, fun_ptr), this) = match **fun_name {
                InstanceVar(ref lhs, ref name) => {
                    let (obj_var, obj_ptr) = try!(eval_exp(lhs, state.clone()));
                    let state_clone = state.clone();
                    (try!(instance_var_eval!(obj_var.clone(), obj_ptr.clone(), name, state_clone)), obj_ptr.map(|x| (obj_var, x)))
                }
                _ => (try!(eval_exp(fun_name, state.clone())), None)
            };

            // Create vector of arguments, evaluated to JsVars.
            let mut args = Vec::new();
            for exp in arg_exps {
                args.push(try!(eval_exp(exp, state.clone())));
            }

            let js_fn_struct = match fun_ptr {
                Some(JsPtrEnum::JsFn(fun)) => fun,
                Some(JsPtrEnum::NativeFn(func)) => return func.call(state.clone(), this, args),
                Some(_) =>
                    return Err(JsError::TypeError(format!("{:?} is not a function", fun_name))),
                None => match state.borrow_mut().load(&fun_binding.binding) {
                    Ok((_, Some(JsPtrEnum::JsFn(fun)))) => fun,
                    Ok(_) =>
                        return Err(JsError::TypeError(format!("{:?} is not a function", fun_name))),
                    Err(_) =>
                        return Err(JsError::ReferenceError(format!("{:?} is not defined", fun_name))),
                }
            };

            match js_fn_struct.name {
                Some(_) => state.borrow_mut().push_scope(e),
                None => try!(state.borrow_mut().push_closure_scope(&fun_binding.unique))
            };

            for param in js_fn_struct.params {
                let mut arg = if args.is_empty() {
                    scalar(JsUndef)
                } else {
                    args.remove(0)
                };

                arg.0.binding = Binding::new(param.to_owned());
                state.borrow_mut().alloc(arg.0, arg.1)
                .expect("Unable to store function argument in scope");
            }

            let (_, v) = try!(eval_stmt_block(&js_fn_struct.stmt, state.clone()));

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
            state.borrow_mut().pop_scope(returning_closure, false)
                .expect("Unable to clear scope for function");

            Ok(v.unwrap_or(scalar(JsUndef)))
        }

        // function([param1, params]) { body }
        // function opt_binding([param1, params]) { body }
        &Defun(ref opt_binding, ref params, ref body) => {
            let js_fun = JsFnStruct::new(opt_binding, params, body);

            let var = if let &Some(ref s) = opt_binding {
                JsVar::bind(s, JsPtr(JsPtrTag::JsFn { name: opt_binding.clone() }))
            } else {
                JsVar::new(JsPtr(JsPtrTag::JsFn { name: None }))
            };

            if let Err(e) = state.borrow_mut().alloc(var.clone(), Some(JsPtrEnum::JsFn(js_fun.clone()))) {
                return Err(JsError::GcError(e));
            }

            Ok((var, Some(JsPtrEnum::JsFn(js_fun))))
        },

        // var.binding
        &InstanceVar(ref instance_exp, ref var) => {
            // TODO: this needs better type-reasoning and errors
            let (instance_var, var_ptr) = try!(eval_exp(instance_exp, state.clone()));
            let state_clone = state.clone();
            instance_var_eval!(instance_var, var_ptr, var, state_clone)
        },

        &Float(f) => Ok(scalar(JsType::JsNum(f))),
        &KeyAccessor(ref obj, ref key) => {
            let (var, ptr) = try!(eval_exp(key, state.clone()));
            let string = ptr.map(|p| p.as_string()).unwrap_or(var.t.as_string());
            eval_exp(&InstanceVar(obj.clone(), string), state)
        }
        &LogNot(ref exp) => Ok(scalar(JsBool(!try!(eval_exp(exp, state.clone())).0.as_bool()))),
        &Neg(ref exp) => Ok(scalar(JsNum(-try!(eval_exp(exp, state.clone())).0.as_number()))),
        &Null => Ok(scalar(JsNull)),
        &Pos(ref exp) => Ok(scalar(JsNum(try!(eval_exp(exp, state.clone())).0.as_number()))),

        &PostDec(ref exp) => eval_float_post_op!(exp, f, f - 1.0, state),
        &PostInc(ref exp) => eval_float_post_op!(exp, f, f + 1.0, state),
        &PreDec(ref exp)  => eval_float_pre_op!(exp, f, f - 1.0, state),
        &PreInc(ref exp)  => eval_float_pre_op!(exp, f, f + 1.0, state),

        &NewObject(ref proto, ref args) => {
            eval_exp(&Call(proto.clone(), args.clone()), state.clone())
        }
        &Object(ref fields) => {
            let mut kv_tuples = Vec::new();
            for &(ref key, ref val) in fields {
                let f_key = js_str_key(&key);
                // TODO: handle obj as key/value pair

                let (mut f_var, f_ptr) = try!(eval_exp(&*val, state.clone()));

                let old_binding = f_var.unique.clone();
                f_var.mangle(&key);
                let _ = state.borrow_mut().rename_closure(&old_binding, &f_var.unique);

                kv_tuples.push((f_key, f_var, f_ptr));
            }

            let mut state_ref = state.borrow_mut();
            let obj = JsObjStruct::new(None, "Object", kv_tuples, &mut *(state_ref.alloc_box.borrow_mut()));

            Ok((JsVar::new(JsPtr(JsPtrTag::JsObj)), Some(JsPtrEnum::JsObj(obj))))
        }

        &Str(ref s) => {
            let var = JsVar::new(JsPtr(JsPtrTag::JsStr));
            match unescape(s) {
                Some(s) =>  {
                    let ptr = Some(JsPtrEnum::JsStr(JsStrStruct::new(&s)));
                    try!(state.borrow_mut().alloc(var.clone(), ptr.clone()));
                    Ok((var, ptr))
                },
                None => {
                    Err(JsError::ParseError(String::from("invalid string")))
                }
            }
        }
        &TypeOf(ref e) =>
            Ok((
                    JsVar::new(JsPtr(JsPtrTag::JsStr)),
                    Some(JsPtrEnum::JsStr(JsStrStruct::new(&try!(eval_exp(e, state.clone())).0.type_of())))
            )),
        &Undefined => Ok(scalar(JsUndef)),
        &Var(ref var_binding) => {
            match state.borrow_mut().load(&Binding::new(var_binding.clone())) {
                Ok((var, ptr)) => Ok((var, ptr)),
                Err(_) => Err(JsError::undefined(var_binding)),
            }
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
