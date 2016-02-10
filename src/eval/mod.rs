#[macro_use]
mod macros;

use coerce::{AsBool,AsNumber};

use french_press::ScopeManager;
use js_types::binding::Binding;
use js_types::js_var::{JsVar, JsType};
use js_types::js_var::JsType::*;

use jsrs_parser::lalr::parse_Stmt;
use jsrs_common::ast::*;
use jsrs_common::ast::Exp::*;
use jsrs_common::ast::BinOp::*;
use jsrs_common::ast::Stmt::*;

pub fn eval_string(string: &str, state: &mut ScopeManager) -> JsVar {
    match parse_Stmt(string) {
        Ok(stmt) => eval_stmt(&stmt, state),
        //Err(e) => JsError(format!("{:?}", e))
        Err(_) => unimplemented!()
    }
    //eval_stmt(parse_Stmt(string).unwrap(), state)
}

pub fn eval_stmt(s: &Stmt, mut state: &mut ScopeManager) -> JsVar {
    match *s {
        Assign(ref var_string, ref exp) => {
            // TODO: this is a hack to return the value properly, which should be changed once we
            // stop using HashMap to store state.
            let mut var = eval_exp(exp, state);
            let cloned = var.clone();
            var.binding = Binding::new(var_string.clone());
            match state.alloc(var, None) {
                Ok(_) => (),
                e @ Err(_) => println!("{:?}", e),
            }
            cloned
        },
        BareExp(ref exp) => eval_exp(exp, &mut state),
        Decl(ref var_string, ref exp) => {
            let mut var = eval_exp(exp, state);
            var.binding = Binding::new(var_string.clone());
            // TODO: use value
            match state.alloc(var.clone(), None) {
                Ok(_) => var,
                e @ Err(_) => panic!("{:?}", e),
            }
        },
        If(ref condition, ref if_block, ref else_block) => {
            if eval_exp(&condition, state).as_bool() {
                eval_stmt(&*if_block, state)
            } else {
                if let Some(ref block) = *else_block {
                    eval_stmt(&*block, state)
                } else {
                    JsVar::new(JsType::JsUndef)
                }
            }
        },
        Ret(_) => panic!("unimplemented: ret statement"),
        Seq(ref s1, ref s2) => {
            let _exp = eval_stmt(&*s1, &mut state);
            eval_stmt(&*s2, &mut state)
        },
        While(ref condition, ref block) => {
            let mut ret_val = JsVar::new(JsUndef);
            loop {
                if eval_exp(&condition, state).as_bool() {
                    ret_val = eval_stmt(&*block, state)
                } else {
                    return ret_val
                }
            }
        }
    }
}

pub fn eval_exp(e: &Exp, mut state: &mut ScopeManager) -> JsVar {
    match e {
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
        &Call(_, _) => {
            unimplemented!()
            //// TODO: create scope with arguments
            //let fun_name = eval_exp(fun_exp, state);
            //match fun_name {
            //    JsFunction(_, _, stmt) => eval_stmt(&*stmt, state),
            //    _ => panic!("TypeError: {} is not a function.", fun_name)
            //}
        },
        &Defun(_, _, _) => {
            unimplemented!()
            // TODO unimpl
            //if let Some(ref var) = *opt_var {
            //    let f = JsFunction(var.clone(), params.clone(), (*block).clone());
            //    state.insert(var.clone(), f);
            //    JsUndefined
            //} else {
            //    JsFunction(String::from(""), params.clone(), (*block).clone())
            //}
        },
        &Float(f) => JsVar::new(JsType::JsNum(f)),
        &Neg(ref exp) => JsVar::new(JsNum(-eval_exp(exp, state).as_number())),
        &Pos(ref exp) => JsVar::new(JsNum(eval_exp(exp, state).as_number())),

        &PostDec(ref exp) => eval_float_post_op!(exp, f, f - 1.0, state),
        &PostInc(ref exp) => eval_float_post_op!(exp, f, f + 1.0, state),
        &PreDec(ref exp)  => eval_float_pre_op!(exp, f, f - 1.0, state),
        &PreInc(ref exp)  => eval_float_pre_op!(exp, f, f + 1.0, state),

        &NewObject(_, _) => unimplemented!(),
        &Object(_) => unimplemented!(),
        &Undefined => JsVar::new(JsUndef),
        &Var(ref var_binding) => {
            match state.load(&Binding::new(var_binding.clone())) {
                Ok((var, _)) => var,
                _ => panic!(format!("ReferenceError: {} is not defined", var_binding))
            }
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
