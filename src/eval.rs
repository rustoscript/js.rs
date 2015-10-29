use std;
use std::f64::NAN;

use jsrs_parser::arith::parse_Exp;
use jsrs_parser::ast::*;
use jsrs_parser::ast::Exp::*;
use jsrs_parser::ast::BinOp::*;

pub fn eval_string(s: &str) -> Exp {
    eval(*parse_Exp(s).unwrap())
}

pub fn eval(e: Exp) -> Exp {
    match e {
        Int(i) => Int(i),
        Float(f) => Float(f),
        BinExp(e1, op, e2) => {
            let eval1 = eval(*e1);
            let eval2 = eval(*e2);
            match op {
                Star => {
                    match (eval1, eval2) {
                        (Int(i1), Int(i2))     => Int(i1 * i2),
                        (Float(f1), Int(i2))   => Float(f1 * i2 as f64),
                        (Int(i1), Float(f2))   => Float(i1 as f64 * f2),
                        (Float(f1), Float(f2)) => Float(f1 * f2),
                        _ => Float(std::f64::NAN)
                    }
                },
                Plus => {
                    match (eval1, eval2) {
                        (Int(i1), Int(i2))     => Int(i1 + i2),
                        (Float(f1), Int(i2))   => Float(f1 + i2 as f64),
                        (Int(i1), Float(f2))   => Float(i1 as f64 + f2),
                        (Float(f1), Float(f2)) => Float(f1 + f2),
                        _ => Float(std::f64::NAN)
                    }
                },
                Minus => {
                    match (eval1, eval2) {
                        (Int(i1), Int(i2))     => Int(i1 - i2),
                        (Float(f1), Int(i2))   => Float(f1 - i2 as f64),
                        (Int(i1), Float(f2))   => Float(i1 as f64 - f2),
                        (Float(f1), Float(f2)) => Float(f1 - f2),
                        _ => Float(std::f64::NAN)
                    }
                },
            }
        }
    }
}
