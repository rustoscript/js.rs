use std;
use std::f64::NAN;

use jsrs_parser::lalr::parse_Exp;
use jsrs_parser::lalr::parse_Stmt;
use jsrs_parser::ast::*;
use jsrs_parser::ast::Exp::*;
use jsrs_parser::ast::BinOp::*;

pub fn eval_string(s: &str) -> Exp {
    eval(parse_Exp(s).unwrap())
}

pub fn eval(e: Exp) -> Exp {
    match e {
        Float(f) => Float(f),
        BinExp(e1, op, e2) => {
            let eval1 = eval(*e1);
            let eval2 = eval(*e2);
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
        // TODO:
        Var(_) => Float(std::f64::NAN),
    }
}
