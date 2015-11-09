use std;
use std::f64::NAN;
use std::collections::HashMap;

use value::JsValue;
use value::JsValue::*;

use jsrs_parser::lalr::parse_Stmt;
use jsrs_parser::ast::*;
use jsrs_parser::ast::Exp::*;
use jsrs_parser::ast::BinOp::*;
use jsrs_parser::ast::Stmt::*;

pub fn eval_string(string: &str, state: &mut HashMap<String, JsValue>)
        -> Result<JsValue, String> {
    match parse_Stmt(string) {
        Ok(stmt) => match eval_stmt(stmt, state) {
            Ok(value) => Ok(value),
            Err(s) => Err(s)
        },
        Err(parse_error) => Err(format!("SyntaxError: {:?}", parse_error))
    }
    //parse_Stmt(string).map(|stmt| eval_stmt(stmt, state))
        //.map_err(|parse_error| format!("SyntaxError: {:?}", parse_error))
}

pub fn eval_stmt(s: Stmt, mut state: &mut HashMap<String, JsValue>)
        -> Result<JsValue, String> {
    match s {
        Assign(var_string, exp) => {
            let eval = eval_exp(exp, state);
            state.insert(var_string, eval);
            Ok(JsUndefined)
        },
        BareExp(exp) => eval_exp(exp, &mut state),
        Decl(var_string, exp) => {
            let eval = eval_exp(exp, state);
            state.insert(var_string, eval);
            Ok(JsUndefined)
        },
        Seq(s1, s2) => {
            let _exp = eval_stmt(*s1, &mut state);
            Ok(eval_stmt(*s2, &mut state))
        }
    }
}

pub fn eval_exp(e: Exp, mut state: &mut HashMap<String, JsValue>) -> JsValue {
    match e {
        BinExp(e1, op, e2) => {
            let val1 = eval_exp(*e1, state);
            let val2 = eval_exp(*e2, state);

            match op {
                Minus => eval_float_binop!(val1, val2, f1, f2, f1 - f2),
                Plus  => eval_float_binop!(val1, val2, f1, f2, f1 + f2),
                Slash => eval_float_binop!(val1, val2, f1, f2, f1 / f2),
                Star  => eval_float_binop!(val1, val2, f1, f2, f1 * f2),
            }
        }
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
                _ => panic!("ReferenceError: {} is not defined", var)
            }
        }
    }
}
