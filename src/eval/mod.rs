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
        Ok(stmt) => eval_stmt(stmt, state),
        Err(e) => JsError(format!("{:?}", e))
    }
    //eval_stmt(parse_Stmt(string).unwrap(), state)
}

pub fn eval_stmt(s: Stmt, mut state: &mut HashMap<String, JsValue>) -> JsValue {
    match s {
        Assign(var_string, exp) => {
            // TODO: this is a hack to return the value properly, which should be changed once we
            // stop using HashMap to store state.
            let val = eval_exp(exp, state);
            let cloned = val.clone();
            state.insert(var_string, val);
            cloned
        },
        BareExp(exp) => eval_exp(exp, &mut state),
        Decl(var_string, exp) => {
            let val = eval_exp(exp, state);
            state.insert(var_string, val);
            JsUndefined
        },
        If(_, _, _) => panic!("unimplemented: if statement"),
        Ret(_) => panic!("unimplemented: ret statement"),
        Seq(s1, s2) => {
            let _exp = eval_stmt(*s1, &mut state);
            eval_stmt(*s2, &mut state)
        },
        While(_, _) => panic!("unimplemented: while statement"),
    }
}

pub fn eval_exp(e: Exp, mut state: &mut HashMap<String, JsValue>) -> JsValue {
    match e {
        BinExp(e1, op, e2) => {
            let val1 = eval_exp(*e1, state);
            let val2 = eval_exp(*e2, state);

            match op {
                And   => panic!("unimplemented: and"),
                Ge    => panic!("unimplemented: ge"),
                Gt    => panic!("unimplemented: gt"),
                Eql   => panic!("unimplemented: eql"),
                Le    => panic!("unimplemented: le"),
                Lt    => panic!("unimplemented: lt"),
                Neq   => panic!("unimplemented: neq"),

                Minus => eval_float_binop!(val1, val2, f1, f2, f1 - f2),
                Or    => panic!("unimplemented: or"),
                Plus  => eval_float_binop!(val1, val2, f1, f2, f1 + f2),
                Slash => eval_float_binop!(val1, val2, f1, f2, f1 / f2),
                Star  => eval_float_binop!(val1, val2, f1, f2, f1 * f2),
            }
        }
        Bool(b) => JsBool(b),
        Call(_, _) => panic!("unimplemented: call"),
        Defun(_, _, _) => panic!("unimplemented: defun"),
        Float(f) => JsNumber(f),
        Neg(exp) => eval_float_sign!("Neg", exp, f, -f, state),
        Pos(exp) => eval_float_sign!("Pos", exp, f, f, state),
        PostDec(exp) => eval_float_post_op!(exp, f, f - 1.0, state),
        PostInc(exp) => eval_float_post_op!(exp, f, f + 1.0, state),
        PreDec(exp) => eval_float_pre_op!(exp, f, f - 1.0, state),
        PreInc(exp) => eval_float_pre_op!(exp, f, f + 1.0, state),
        Undefined => JsUndefined,
        Var(var) => {
            match state.get(&var) {
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
