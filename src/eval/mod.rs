#[macro_use]
mod macros;

use coerce::{AsBool,AsNumber};

use french_press::ScopeManager;
use french_press::alloc::AllocBox;
use js_types::binding::Binding;
use js_types::js_fn::JsFnStruct;
use js_types::js_obj::JsObjStruct;
use js_types::js_str::JsStrStruct;
use js_types::js_var::{JsVar, JsType, JsPtrEnum, JsKey, JsPtrTag};
use js_types::js_var::JsType::*;

use jsrs_parser::lalr::parse_Stmt;
use jsrs_common::ast::*;
use jsrs_common::ast::Exp::*;
use jsrs_common::ast::BinOp::*;
use jsrs_common::ast::Stmt::*;

type VarWithPtr = (JsVar, Option<JsPtrEnum>);

// Helper to avoid repeating this everywhere
fn scalar(v: JsType) -> (JsVar, Option<JsPtrEnum>) {
    (JsVar::new(v), None)
}

/// Evaluate a string containing some JavaScript statements (or sequences of statements).
/// Returns a JsVar which is the return value of those statements.
pub fn eval_string(string: &str, state: &mut ScopeManager) -> JsVar {
    match parse_Stmt(string) {
        Ok(stmt) => {
            eval_stmt(&stmt, state).0
        }
        Err(e) => panic!("parse error: {:?}", e),
    }
}

/// Evaluate a single JS statement (which may be a block or sequence of statements).
/// Returns tuple of (evaluated final value, return value), where return value requires that
/// `return` be used to generate it.
pub fn eval_stmt(s: &Stmt, mut state: &mut ScopeManager) -> (JsVar, Option<JsVar>) {
    match *s {
        // var_string = exp;
        Assign(ref var_string, ref exp) => {
            let (mut js_var, js_ptr) = eval_exp(exp, state);
            js_var.binding = Binding::new(var_string.clone());

            // Clone the js_var to store into the ScopeManager
            let cloned = js_var.clone();
            match state.alloc(cloned, js_ptr) {
                Ok(_) => (),
                e @ Err(_) => println!("{:?}", e),
            }

            (js_var, None)
        },

        // exp;
        BareExp(ref exp) => (eval_exp(exp, &mut state).0, None),

        // var var_string = exp
        Decl(ref var_string, ref exp) => {
            let (mut js_var, js_ptr) = eval_exp(exp, state);
            js_var.binding = Binding::new(var_string.clone());
            match state.alloc(js_var.clone(), js_ptr) {
                Ok(_) => (js_var, None),
                e @ Err(_) => panic!("{:?}", e),
            }
        },

        // Empty statement (?)
        Empty => (JsVar::new(JsUndef), None),

        // if (condition) { if_block } else { else_block }
        If(ref condition, ref if_block, ref else_block) => {
            // evaluate expression
            if eval_exp(&condition, state).0.as_bool() {
                // condition = true, evaluate if-block.
                eval_stmt(&*if_block, state)
            } else {
                // condition = false
                // evaluate else-block if it exists, otherwise return undefined.
                if let Some(ref block) = *else_block {
                    eval_stmt(&*block, state)
                } else {
                    (JsVar::new(JsUndef), None)
                }
            }
        },

        // return exp
        Ret(ref exp) => {
            let js_var = eval_exp(&exp, &mut state).0;
            (js_var.clone(), Some(js_var))
        }

        // a sequence of any two expressions
        Seq(ref s1, ref s2) => {
            eval_stmt(&*s1, &mut state);
            eval_stmt(&*s2, &mut state)
        },

        // while (condition) { block }
        While(ref condition, ref block) => {
            let mut ret_val = None;
            loop {
                if eval_exp(&condition, state).0.as_bool() {
                    // TODO: check to see if a return stmt has been reached.
                    let (_, v) = eval_stmt(&*block, state);
                    ret_val = v;
                } else {
                    // condition is no longer true, return a return value
                    return (JsVar::new(JsUndef), ret_val);
                }
            }
        }
    }
}

/// Evaluate an expression into a JsVar.
pub fn eval_exp(e: &Exp, mut state: &mut ScopeManager) -> (JsVar, Option<JsPtrEnum>) {
    match e {
        // e1 [op] e2
        &BinExp(ref e1, ref op, ref e2) => {
            let val1 = eval_exp(e1, state).0;
            let val2 = eval_exp(e2, state).0;

            match *op {
                And => scalar(JsBool(val1.as_bool() && val2.as_bool())),
                Or  => scalar(JsBool(val1.as_bool() || val2.as_bool())),

                Ge  => scalar(JsBool(val1.as_bool() >= val2.as_bool())),
                Gt  => scalar(JsBool(val1.as_bool() >  val2.as_bool())),
                Le  => scalar(JsBool(val1.as_bool() <= val2.as_bool())),
                Lt  => scalar(JsBool(val1.as_bool() <  val2.as_bool())),
                Neq => scalar(JsBool(val1.as_bool() != val2.as_bool())),
                Eql => scalar(JsBool(val1.as_bool() == val2.as_bool())),

                Minus => scalar(JsNum(val1.as_number() - val2.as_number())),
                Plus  => scalar(JsNum(val1.as_number() + val2.as_number())),
                Slash => scalar(JsNum(val1.as_number() / val2.as_number())),
                Star  => scalar(JsNum(val1.as_number() * val2.as_number())),
            }
        }
        &Bool(b) => scalar(JsBool(b)),

        // fun_name([arg_exp1, arg_exps])
        &Call(ref fun_name, ref arg_exps) => {
            let fun_binding = eval_exp(fun_name, state).0;

            // Create vector of arguments, evaluated to JsVars.
            let mut args = Vec::new();
            for exp in arg_exps {
                args.push(eval_exp(exp, state));
            }

            match state.load(&fun_binding.binding) {
                Ok((_, Some(JsPtrEnum::JsFn(js_fn_struct)))) => {
                    state.push_scope(e);

                    for param in js_fn_struct.params {
                        let mut arg = if args.is_empty() {
                            scalar(JsUndef)
                        } else {
                            args.remove(0)
                        };

                        arg.0.binding = Binding::new(param.to_owned());
                        state.alloc(arg.0, arg.1)
                            .expect("Unable to store function argument in scope");
                    }

                    let (_, v) = eval_stmt(&js_fn_struct.stmt, state);

                    // Should we yield here? Not sure, so for now it doesn't
                    state.pop_scope(false).expect("Unable to clear scope for function");
                    // TODO handle obj
                    v.map(|x| (x, None)).unwrap_or(scalar(JsUndef))
                }
                Ok(_) => panic!("Invalid call object"),
                _ => panic!("ReferenceError: {:?} is not defined", fun_name)
            }
        },

        // function([param1, params]) { body }
        // function opt_binding([param1, params]) { body }
        &Defun(ref opt_binding, ref params, ref body) => {
            if let &Some(ref binding) = opt_binding {
                let js_fun = JsFnStruct::new(opt_binding, params, &**body);
                state.alloc(JsVar::bind(binding.to_owned(), JsPtr(JsPtrTag::JsFn)),
                    Some(JsPtrEnum::JsFn(js_fun.clone())))
                    .expect("Error storing function into state");
                (JsVar::bind(binding.to_owned(), JsPtr(JsPtrTag::JsFn)),
                Some(JsPtrEnum::JsFn(js_fun)))
            } else {
                panic!("functions without bindings are not yet supported.")
            }
        },

        // var.binding
        &InstanceVar(ref var, ref binding) => {
            unimplemented!();
        },

        &Method(_, _, _) => {
            unimplemented!();
        },

        &Null => {
            unimplemented!();
        },

        &Float(f) => scalar(JsType::JsNum(f)),
        &Neg(ref exp) => scalar(JsNum(-eval_exp(exp, state).0.as_number())),
        &Pos(ref exp) => scalar(JsNum(eval_exp(exp, state).0.as_number())),

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
                let f_exp = eval_exp(&*f.1, state).0;
                kv_tuples.push((f_key, f_exp, None));
            }
            let obj = JsObjStruct::new(None, "", kv_tuples, &mut AllocBox::new());
            (JsVar::new(JsPtr(JsPtrTag::JsObj)), Some(JsPtrEnum::JsObj(obj)))
        },

        &Undefined => scalar(JsUndef),
        &Var(ref var_binding) => {
            state.load(&Binding::new(var_binding.clone()))
                .expect("ReferenceError: {} is not defined")
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::hash_set::HashSet;
    use french_press::init_gc;
    use js_types::js_var::JsType;
    use js_types::binding::Binding;

    #[test]
    fn test_eval_literals() {
        let mut state = init_gc();
        assert_eq!(JsType::JsNum(5.0f64), eval_string("5.0;\n", &mut state).t);
        assert_eq!(JsType::JsNum(0.0f64), eval_string("0.0;\n", &mut state).t);
        assert_eq!(JsType::JsUndef, eval_string("undefined;\n", &mut state).t);
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
        let mut state = init_gc();
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
        let mut state = init_gc();
        assert_eq!(JsType::JsNum(6.0f64),  eval_string("2.0 + 4.0;\n", &mut state).t);
        assert_eq!(JsType::JsNum(0.5f64),  eval_string("2.0 / 4.0;\n", &mut state).t);
        assert_eq!(JsType::JsNum(-2.0f64), eval_string("2.0 - 4.0;\n", &mut state).t);
        assert_eq!(JsType::JsNum(8.0f64),  eval_string("2.0 * 4.0;\n", &mut state).t);
    }
}
