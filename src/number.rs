use jsrs_common::ast::*;
use jsrs_common::ast::BinOp::*;
use jsrs_common::types::coerce::{AsBool,AsNumber};
use jsrs_common::types::js_var::{JsVar, JsType};
use jsrs_common::types::js_var::JsType::*;


pub fn eval_binop(op: &BinOp, val1: JsVar, val2: JsVar) -> JsType {
    match *op {
        And => JsBool(val1.as_bool() && val2.as_bool()),
        Or  => JsBool(val1.as_bool() || val2.as_bool()),

        Ge  => JsBool(val1.as_bool() >= val2.as_bool()),
        Gt  => JsBool(val1.as_bool() >  val2.as_bool()),
        Le  => JsBool(val1.as_bool() <= val2.as_bool()),
        Lt  => JsBool(val1.as_bool() <  val2.as_bool()),

        // TODO: equality?
        Neq => JsBool(val1.as_bool() != val2.as_bool()),
        Eql => JsBool(val1.as_bool() == val2.as_bool()),

        EqlStrict => JsBool(val1 == val2),
        NeqStrict => JsBool(val1 != val2),

        Minus => JsNum(val1.as_number() - val2.as_number()),
        Plus  => JsNum(val1.as_number() + val2.as_number()),
        Slash => JsNum(val1.as_number() / val2.as_number()),
        Star  => JsNum(val1.as_number() * val2.as_number()),
    }
}
