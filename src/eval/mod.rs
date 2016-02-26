#[macro_use]
mod macros;

use coerce::{AsBool,AsNumber};

use french_press::ScopeManager;
use french_press::alloc::AllocBox;
use js_types::binding::Binding;
use js_types::js_fn::JsFnStruct;
use js_types::js_obj::JsObjStruct;
use js_types::js_str::JsStrStruct;
use js_types::js_var::{JsVar, JsType, JsPtrEnum, JsKey, JsKeyEnum};
use js_types::js_var::JsType::*;

use jsrs_parser::lalr::parse_Stmt;
use jsrs_common::ast::*;
use jsrs_common::ast::Exp::*;
use jsrs_common::ast::BinOp::*;
use jsrs_common::ast::Stmt::*;

/// Evaluate a string containing some JavaScript statements (or sequences of statements).
/// Returns a JsVar which is the return value of those statements.
pub fn eval_string(string: &str, state: &mut ScopeManager) -> JsVar {
    match parse_Stmt(string) {
        Ok(stmt) => {
            eval_stmt(&stmt, state).0
        }
        Err(_) => panic!("parse error"),
    }
}

/// Evaluate a single JS statement (which may be a block or sequence of statements).
/// Returns tuple of (evaluated final value, return value), where return value requires that
/// `return` be used to generate it.
pub fn eval_stmt(s: &Stmt, mut state: &mut ScopeManager) -> (JsVar, Option<JsVar>) {
    match *s {
        // var_string = exp;
        Assign(ref var_string, ref exp) => {
            let mut js_var = eval_exp(exp, state);
            js_var.binding = Binding::new(var_string.clone());

            // Clone the js_var to store into the ScopeManager
            let cloned = js_var.clone();
            match state.alloc(cloned, None) {
                Ok(_) => (),
                e @ Err(_) => println!("{:?}", e),
            }

            (js_var, None)
        },

        // exp;
        BareExp(ref exp) => (eval_exp(exp, &mut state), None),

        // var var_string = exp
        Decl(ref var_string, ref exp) => {
            let mut js_var = eval_exp(exp, state);
            js_var.binding = Binding::new(var_string.clone());
            // TODO: must handle JsPtrEnum here.
            match state.alloc(js_var.clone(), None) {
                Ok(_) => (js_var, None),
                e @ Err(_) => panic!("{:?}", e),
            }
        },

        // if (condition) { if_block } else { else_block }
        If(ref condition, ref if_block, ref else_block) => {
            // evaluate expression
            if eval_exp(&condition, state).as_bool() {
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
            let js_var = eval_exp(&exp, &mut state);
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
                if eval_exp(&condition, state).as_bool() {
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
pub fn eval_exp(e: &Exp, mut state: &mut ScopeManager) -> JsVar {
    match e {
        // e1 [op] e2
        &BinExp(ref e1, ref op, ref e2) => {
            let val1 = eval_exp(e1, state);
            let val2 = eval_exp(e2, state);

            match *op {
                And => JsVar::new(JsBool(val1.as_bool() && val2.as_bool())),
                Or  => JsVar::new(JsBool(val1.as_bool() || val2.as_bool())),

                Ge  => JsVar::new(JsBool(val1.as_bool() >= val2.as_bool())),
                Gt  => JsVar::new(JsBool(val1.as_bool() >  val2.as_bool())),
                Le  => JsVar::new(JsBool(val1.as_bool() <= val2.as_bool())),
                Lt  => JsVar::new(JsBool(val1.as_bool() <  val2.as_bool())),
                Neq => JsVar::new(JsBool(val1.as_bool() != val2.as_bool())),
                Eql => JsVar::new(JsBool(val1.as_bool() == val2.as_bool())),

                Minus => JsVar::new(JsNum(val1.as_number() - val2.as_number())),
                Plus  => JsVar::new(JsNum(val1.as_number() + val2.as_number())),
                Slash => JsVar::new(JsNum(val1.as_number() / val2.as_number())),
                Star  => JsVar::new(JsNum(val1.as_number() * val2.as_number())),
            }
        }
        &Bool(b) => JsVar::new(JsBool(b)),

        // fun_name([arg_exp1, arg_exps])
        &Call(ref fun_name, ref arg_exps) => {
            let fun_binding = eval_exp(fun_name, state);

            // Create vector of arguments, evaluated to JsVars.
            let mut args = Vec::new();
            for exp in arg_exps {
                args.push(eval_exp(exp, state));
            }

            match state.load(&fun_binding.binding) {
                Ok((_, Some(JsPtrEnum::JsFn(js_fn_struct)))) => {
                    state.push_scope();

                    for param in js_fn_struct.params {
                        let mut arg = if args.is_empty() {
                            JsVar::new(JsUndef)
                        } else {
                            args.remove(0)
                        };

                        arg.binding = Binding::new(param.to_owned());
                        state.alloc(arg, None)
                            .expect("Unable to store function argument in scope");
                    }

                    let (_, v) = eval_stmt(&js_fn_struct.stmt, state);

                    // Should we yield here? Not sure, so for now it doesn't
                    state.pop_scope(false).expect("Unable to clear scope for function");
                    v.unwrap_or(JsVar::new(JsUndef))
                }
                Ok(_) => panic!("Invalid call object"),
                _ => panic!("ReferenceError: {} is not defined")
            }
        },

        // function([param1, params]) { body }
        // function opt_binding([param1, params]) { body }
        &Defun(ref opt_binding, ref params, ref body) => {
            if let &Some(ref binding) = opt_binding {
                let js_fun = JsFnStruct::new(opt_binding, params, &**body);
                let var = JsVar::bind(binding.clone(), JsPtr);
                state.alloc(var.clone(), Some(JsPtrEnum::JsFn(js_fun)))
                    .expect("error storing function into state");
                var
            } else {
                panic!("functions without bindings are not yet supported.")
            }
        },
        &Float(f) => JsVar::new(JsType::JsNum(f)),
        &Neg(ref exp) => JsVar::new(JsNum(-eval_exp(exp, state).as_number())),
        &Pos(ref exp) => JsVar::new(JsNum(eval_exp(exp, state).as_number())),

        &PostDec(ref exp) => eval_float_post_op!(exp, f, f - 1.0, state),
        &PostInc(ref exp) => eval_float_post_op!(exp, f, f + 1.0, state),
        &PreDec(ref exp)  => eval_float_pre_op!(exp, f, f - 1.0, state),
        &PreInc(ref exp)  => eval_float_pre_op!(exp, f, f + 1.0, state),

        &NewObject(_, _) => unimplemented!(),
        &Object(ref fields) => {
            let mut kv_tuples = Vec::new();
            for f in fields {
                let f_key = JsKey::new(JsKeyEnum::JsStr(JsStrStruct::new(&f.0)));
                let f_exp = eval_exp(&*f.1, state);
                kv_tuples.push((f_key, f_exp, None));
            }
            JsObjStruct::new(None, "", kv_tuples, &mut AllocBox::new());
            JsVar::new(JsPtr)
        },

        &Undefined => JsVar::new(JsUndef),
        &Var(ref var_binding) => {
            state.load(&Binding::new(var_binding.clone()))
                .expect("ReferenceError: {} is not defined").0
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
