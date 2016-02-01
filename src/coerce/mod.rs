use std::f64::NAN;

use french_press::js_types::js_type::JsVar;
use french_press::js_types::js_type::JsType::*;


pub trait AsBool {
    fn as_bool(&self) -> bool;
}

impl AsBool for JsVar {
    fn as_bool(&self) -> bool {
        match self.t {
            JsBool(b) => b,
            JsUndef=> false,
            JsNull => false,
            JsNum(n) =>
                if n == 0.0f64 || n == -0.0f64 || n.is_nan() {
                    false
                } else {
                    true
                },
            JsPtr => true, // TODO - this is incorrect
            //JsString(ref s) =>
            //    if s.len() == 0 {
            //        false
            //    } else {
            //        true
            //    },
            //JsSymbol(_) => true,
            //JsObject | JsError(_) | JsFunction(_, _, _) => true,
        }
    }
}

pub trait AsNumber {
    fn as_number(&self) -> f64;
}

impl AsNumber for JsVar {
    fn as_number(&self) -> f64 {
        match self.t {
            JsBool(b) => if b { 1f64 } else { 0f64 },
            JsUndef => NAN,
            JsNull => 0f64,
            JsNum(n) => n,
            JsPtr => NAN,
            //JsString(ref s) =>
            //    if s.len() == 0 {
            //        JsNumber(0f64)
            //    } else {
            //        let num = s.parse();
            //        match num {
            //            Ok(n)  => JsNumber(n),
            //            Err(_) => JsNumber(NAN),
            //        }
            //    },
            //JsSymbol(_) => panic!("Cannot convert a Symbol to a number."),
            //JsObject | JsError(_) | JsFunction(_, _, _) => JsNumber(NAN),
        }
    }
}
