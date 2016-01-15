use french_press::js_types::js_type::{JsVar, JsType};
use french_press::js_types::js_type::JsType::*;

pub trait AsBool {
    fn as_bool(self) -> bool;
}

impl AsBool for JsVar {
    fn as_bool(self) -> bool {
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


