use std;
use std::f64::NAN;
use std::collections::HashMap;

#[macro_use]
mod macros;

use value::JsValue;
use value::JsValue::*;

use jsrs_parser::lalr::parse_Stmt;
use jsrs_common::ast::*;
use jsrs_common::ast::Exp::*;
use jsrs_common::ast::BinOp::*;
use jsrs_common::ast::Stmt::*;

pub fn eval_string(string: &str, state: &mut HashMap<String, JsValue>) -> JsValue {
    match parse_Stmt(string) {
        Ok(stmt) => eval_stmt(&stmt, state),
        Err(e) => JsError(format!("{:?}", e))
    }
    //eval_stmt(parse_Stmt(string).unwrap(), state)
}

pub fn eval_stmt(s: &Stmt, mut state: &mut HashMap<String, JsValue>) -> JsValue {
    match *s {
        Assign(ref var_string, ref exp) => {
            // TODO: this is a hack to return the value properly, which should be changed once we
            // stop using HashMap to store state.
            let val = eval_exp(exp, state);
            let cloned = val.clone();
            state.insert(var_string.clone(), val);
            cloned
        },
        BareExp(ref exp) => eval_exp(exp, &mut state),
        Decl(ref var_string, ref exp) => {
            let val = eval_exp(exp, state);
            state.insert(var_string.clone(), val);
            JsUndefined
        },
        If(ref condition, ref if_block, ref else_block) => {
            if let JsBool(b) = eval_exp(&condition, state).as_bool() {
                if b {
                    eval_stmt(&*if_block, state)
                } else {
                    if let Some(ref block) = *else_block {
                        eval_stmt(&*block, state)
                    } else {
                        JsUndefined
                    }
                }
            } else {
                panic!("invalid boolean expression");
            }
        },
        Ret(_) => panic!("unimplemented: ret statement"),
        Seq(ref s1, ref s2) => {
            let _exp = eval_stmt(&*s1, &mut state);
            eval_stmt(&*s2, &mut state)
        },
        While(ref condition, ref block) => {
            let mut ret_val = JsUndefined;
            loop {
                if let JsBool(b) = eval_exp(&condition, state).as_bool() {
                    if b {
                        ret_val = eval_stmt(&*block, state)
                    } else {
                        return ret_val
                    }
                } else {
                    panic!("invalid boolean expression");
                }
            }
        }
    }
}

pub fn eval_exp(e: &Exp, mut state: &mut HashMap<String, JsValue>) -> JsValue {
    match e {
        &BinExp(ref e1, ref op, ref e2) => {
            let val1 = eval_exp(e1, state);
            let val2 = eval_exp(e2, state);

            match *op {
                And   => eval_logic!(val1, val2, f1, f2, f1 && f2),
                Or    => eval_logic!(val1, val2, f1, f2, f1 || f2),

                Ge    => eval_cmp!(val1, val2, f1, f2, f1 >= f2),
                Gt    => eval_cmp!(val1, val2, f1, f2, f1 >  f2),
                Le    => eval_cmp!(val1, val2, f1, f2, f1 <= f2),
                Lt    => eval_cmp!(val1, val2, f1, f2, f1 <  f2),
                Neq   => eval_cmp!(val1, val2, f1, f2, f1 != f2),
                Eql   => eval_cmp!(val1, val2, f1, f2, f1 == f2),

                Minus => eval_num_binop!(val1, val2, f1, f2, f1 - f2),
                Plus  => eval_num_binop!(val1, val2, f1, f2, f1 + f2),
                Slash => eval_num_binop!(val1, val2, f1, f2, f1 / f2),
                Star  => eval_num_binop!(val1, val2, f1, f2, f1 * f2),
            }
        }
        &Bool(b) => JsBool(b),
        &Call(ref fun_exp, _) => {
            // TODO: create scope with arguments
            let fun_name = eval_exp(fun_exp, state);
            match fun_name {
                JsFunction(_, _, stmt) => eval_stmt(&*stmt, state),
                _ => panic!("TypeError: {} is not a function.", fun_name)
            }
        },
        &Defun(ref opt_var, ref params, ref block) => {
            if let Some(ref var) = *opt_var {
                let f = JsFunction(var.clone(), params.clone(), (*block).clone());
                state.insert(var.clone(), f);
                JsUndefined
            } else {
                JsFunction(String::from(""), params.clone(), (*block).clone())
            }
        },
        &Float(f) => JsNumber(f),
        &Neg(ref exp) => eval_float_sign!("Neg", exp, f, -f, state),
        &Pos(ref exp) => eval_float_sign!("Pos", exp, f, f, state),
        &PostDec(ref exp) => eval_float_post_op!(exp, f, f - 1.0, state),
        &PostInc(ref exp) => eval_float_post_op!(exp, f, f + 1.0, state),
        &PreDec(ref exp) => eval_float_pre_op!(exp, f, f - 1.0, state),
        &PreInc(ref exp) => eval_float_pre_op!(exp, f, f + 1.0, state),
        &Undefined => JsUndefined,
        &Var(ref var) => {
            match state.get(var) {
                Some(ref a) => (*a).clone(),
                _ => JsError(format!("ReferenceError: {} is not defined", var))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use super::*;
    use value::JsValue::*;

    #[test]
    fn test_eval_literals() {
        let mut state = HashMap::new();
        assert_eq!(JsNumber(5.0f64), eval_string("5.0;\n", &mut state));
        assert_eq!(JsNumber(0.0f64), eval_string("0.0;\n", &mut state));
        assert_eq!(JsUndefined, eval_string("undefined;\n", &mut state));
        assert_eq!(0, state.len());
    }

    // TODO: handle `var` and no `var` separately
    #[test]
    fn test_store_state() {
        let mut state = HashMap::new();
        assert_eq!(JsUndefined, eval_string("var a = 1;\n", &mut state));
        assert_eq!(JsNumber(2.0f64), eval_string("a = 2;\n", &mut state));
        assert_eq!(JsUndefined, eval_string("var b = 3;\n", &mut state));
        assert_eq!(JsNumber(4.0f64), eval_string("c = 4;\n", &mut state));
        assert_eq!(3, state.len());
    }

    #[test]
    fn test_inc_dec() {
        let mut state = HashMap::new();
        assert_eq!(JsUndefined, eval_string("var a = 1;\n", &mut state));
        assert_eq!(&JsNumber(1.0f64), state.get("a").unwrap());

        assert_eq!(JsNumber(1.0f64), eval_string("a++;\n", &mut state));
        assert_eq!(&JsNumber(2.0f64), state.get("a").unwrap());

        assert_eq!(JsNumber(3.0f64), eval_string("++a;\n", &mut state));
        assert_eq!(&JsNumber(3.0f64), state.get("a").unwrap());

        assert_eq!(JsNumber(3.0f64), eval_string("a--;\n", &mut state));
        assert_eq!(&JsNumber(2.0f64), state.get("a").unwrap());

        assert_eq!(JsNumber(1.0f64), eval_string("--a;\n", &mut state));
        assert_eq!(&JsNumber(1.0f64), state.get("a").unwrap());

        assert_eq!(1, state.len());
    }

    #[test]
    fn test_binexp() {
        let mut state = HashMap::new();
        assert_eq!(JsNumber(6.0f64),  eval_string("2.0 + 4.0;\n", &mut state));
        assert_eq!(JsNumber(0.5f64),  eval_string("2.0 / 4.0;\n", &mut state));
        assert_eq!(JsNumber(-2.0f64), eval_string("2.0 - 4.0;\n", &mut state));
        assert_eq!(JsNumber(8.0f64),  eval_string("2.0 * 4.0;\n", &mut state));
        assert_eq!(0, state.len());
    }
}
