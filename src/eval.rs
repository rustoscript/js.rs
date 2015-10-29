use std;
use std::f64::NAN;
use std::collections::HashMap;

use jsrs_parser::lalr::parse_Stmt;
use jsrs_parser::ast::*;
use jsrs_parser::ast::Exp::*;
use jsrs_parser::ast::BinOp::*;
use jsrs_parser::ast::Stmt::*;

pub fn eval_string(string: &str, state: &mut HashMap<String, Exp>) -> Exp {
    eval_stmt(parse_Stmt(string).unwrap(), state)
}

pub fn eval_stmt(s: Stmt, mut state: &mut HashMap<String, Exp>) -> Exp {
    match s {
        Assign(var_string, exp) => {
            let eval = eval_exp(exp, state);
            state.insert(var_string, eval);
            Float(std::f64::NAN)
        },
        Decl(var_string, exp) => {
            let eval = eval_exp(exp, state);
            state.insert(var_string, eval);
            Float(std::f64::NAN)
        },
        BareExp(exp) => eval_exp(exp, &mut state),
    }
}

pub fn eval_exp(e: Exp, mut state: &mut HashMap<String, Exp>) -> Exp {
    match e {
        Float(f) => Float(f),
        BinExp(e1, op, e2) => {
            let eval1 = eval_exp(*e1, state);
            let eval2 = eval_exp(*e2, state);
            match op {
                Star => {
                    match (eval1, eval2) {
                        (Float(f1), Float(f2)) => Float(f1 * f2),
                        _ => Float(std::f64::NAN)
                    }
                },
                Plus => {
                    match (eval1, eval2) {
                        (Float(f1), Float(f2)) => Float(f1 + f2),
                        _ => Float(std::f64::NAN)
                    }
                },
                Minus => {
                    match (eval1, eval2) {
                        (Float(f1), Float(f2)) => Float(f1 - f2),
                        _ => Float(std::f64::NAN)
                    }
                },
                Slash => {
                    match (eval1, eval2) {
                        (Float(f1), Float(f2)) => Float(f1 / f2),
                        _ => Float(std::f64::NAN)
                    }
                },
            }
        }
        Var(var) => {
            if state.contains_key(&var) {
                match state[&var] {
                    Float(f) => Float(f),
                    _ => Float(std::f64::NAN),
                }
            } else {
                Float(std::f64::NAN)
            }
        }
    }
}
