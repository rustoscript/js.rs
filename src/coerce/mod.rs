use std::f64::NAN;

use js_types::js_var::JsVar;
use js_types::js_var::JsPtrEnum;
use js_types::js_var::JsType::*;

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
            JsPtr(_) => true, // TODO - this is incorrect
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
            JsPtr(_) => NAN,
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

fn ptr_as_str(ptr: &JsPtrEnum) -> String {
    match ptr {
        &JsPtrEnum::JsSym(ref s) => format!("Symbol({})", s),
        &JsPtrEnum::JsStr(ref s) => s.text.to_owned(),

        // TODO: Check object's `toString` method
        &JsPtrEnum::JsObj(ref s) => String::from("[object Object]"),

        // TODO: A function's string representation is apparently the string of source code that
        // created it; the AST doesn't currently support this, so we'll need to do some
        // restructuring before we can support this.
        &JsPtrEnum::JsFn(_) => String::from("[function]"),
    }
}

pub trait AsString {
    fn as_string(&self, ptr: Option<&JsPtrEnum>) -> String {
        let s = match self.t {
            JsBool(true) => "true"
            JsBool(false) => "false"
            JsUndef => "undefined",
            JsNull => "null",
            JsNum(n) => return format!("{}", n),
            JsPtr(_) => return ptr_as_str(ptr.expect("Invalid pointer")),
        };

        String::from(s)
    }
}
