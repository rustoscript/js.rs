use std::fmt::{Display, Error, Formatter};
use std::f64::NAN;
use jsrs_common::ast::Stmt;


#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum JsValue {
    // Primitives (cannot be changed)
    JsBool(bool),
    JsUndefined,
    JsNull,
    JsNumber(f64),
    JsString(String),
    JsSymbol(String),
    // Objects
    JsObject,
    // Error value (TODO: more consistent to ECMAScript spec?)
    JsError(String),
    // Special case of Object
    // Need to factor this into a struct (?)
    JsFunction(String, Vec<String>, Box<Stmt>),
}

use self::JsValue::*;

impl Display for JsValue {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match *self {
            JsBool(b) => match b {
                true => write!(fmt, "true"),
                false => write!(fmt, "false")
            },
            JsUndefined => write!(fmt, "undefined"),
            JsNull => write!(fmt, "null"),
            JsNumber(n) => write!(fmt, "{}", n),
            JsString(ref s) => write!(fmt, "{}", s),
            JsSymbol(ref s) => write!(fmt, "{}", s),
            JsObject => write!(fmt, "{{}}"),
            JsError(ref err) => write!(fmt, "{}", err),
            JsFunction(ref var, ref params, ref stmt) =>
                write!(fmt, "function{} ({}) {}", var, params.len(), stmt),
        }
    }
}

impl JsValue {
    pub fn as_bool(self) -> JsValue {
        match self {
            JsBool(_) => self,
            JsUndefined => JsBool(false),
            JsNull => JsBool(false),
            JsNumber(n) =>
                if n == 0.0f64 || n == -0.0f64 || n.is_nan() {
                    JsBool(false)
                } else {
                    JsBool(true)
                },
            JsString(ref s) =>
                if s.len() == 0 {
                    JsBool(false)
                } else {
                    JsBool(true)
                },
            JsSymbol(_) => JsBool(true),
            JsObject | JsError(_) | JsFunction(_, _, _) => JsBool(true),
        }
    }

    pub fn as_number(self) -> JsValue {
        match self {
            JsBool(b) => if b { JsNumber(1f64) } else { JsNumber(0f64) },
            JsUndefined => JsNumber(NAN),
            JsNull => JsNumber(0f64),
            JsNumber(_) => self,
            JsString(ref s) =>
                if s.len() == 0 {
                    JsNumber(0f64)
                } else {
                    let num = s.parse();
                    match num {
                        Ok(n)  => JsNumber(n),
                        Err(_) => JsNumber(NAN),
                    }
                },
            JsSymbol(_) => panic!("Cannot convert a Symbol to a number."),
            JsObject | JsError(_) | JsFunction(_, _, _) => JsNumber(NAN),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::JsValue::*;
    use std::f64::{NAN, INFINITY, NEG_INFINITY};
    use super::*;

    #[test]
    fn jsvalue_display() {
        assert_eq!("true",  format!("{}", JsBool(true)));
        assert_eq!("false", format!("{}", JsBool(false)));
        assert_eq!("undefined", format!("{}", JsUndefined));
        assert_eq!("null",  format!("{}", JsNull));

        assert_eq!("4",    format!("{}", JsNumber(4f64)));
        assert_eq!("-4",   format!("{}", JsNumber(-4f64)));
        assert_eq!("0",    format!("{}", JsNumber(0f64)));
        assert_eq!("NaN",  format!("{}", JsNumber(NAN)));
        assert_eq!("inf",  format!("{}", JsNumber(INFINITY)));
        assert_eq!("-inf", format!("{}", JsNumber(NEG_INFINITY)));

        assert_eq!("", format!("{}", JsString(String::from(""))));
        assert_eq!("string", format!("{}", JsString(String::from("string"))));

        assert_eq!("", format!("{}", JsSymbol(String::from(""))));
        assert_eq!("string", format!("{}", JsSymbol(String::from("string"))));

        assert_eq!("{}", format!("{}", JsObject));
    }

    #[test]
    fn test_as_bool() {
        assert_eq!(JsBool(true ), JsBool(true).as_bool());
        assert_eq!(JsBool(false), JsBool(false).as_bool());

        assert_eq!(JsBool(false), JsUndefined.as_bool());
        assert_eq!(JsBool(false), JsNull.as_bool());
        assert_eq!(JsBool(false), JsNumber(0.0f64).as_bool());
        assert_eq!(JsBool(false), JsNumber(-0.0f64).as_bool());
        assert_eq!(JsBool(false), JsNumber(NAN).as_bool());

        assert_eq!(JsBool(false), JsString(String::from("") ).as_bool());
        assert_eq!(JsBool(true ), JsString(String::from("a")).as_bool());
        assert_eq!(JsBool(true ), JsSymbol(String::from("") ).as_bool());
        assert_eq!(JsBool(true ), JsSymbol(String::from("a")).as_bool());

        assert_eq!(JsBool(true ), JsObject.as_bool());
        assert_eq!(JsBool(true ), JsError(String::from("a")).as_bool());
    }

    fn assert_nan(val: JsValue) {
        if let JsNumber(n) = val {
            assert!(n.is_nan());
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_as_number() {
        assert_eq!(JsNumber(1f64), JsBool(true ).as_number());
        assert_eq!(JsNumber(0f64), JsBool(false).as_number());

        assert_eq!(JsNumber(0f64), JsNull.as_number());
        assert_eq!(JsNumber(0f64), JsNumber(0.0f64 ).as_number());
        assert_eq!(JsNumber(0f64), JsNumber(-0.0f64).as_number());
        assert_eq!(JsNumber(0f64), JsString(String::from("")).as_number());
        //assert_nan(JsSymbol(String::from("") ).as_number());
        //assert_nan(JsSymbol(String::from("a")).as_number());
        assert_nan(JsObject.as_number());
        assert_nan(JsError(String::from("a")).as_number());
        assert_nan(JsUndefined.as_number());
        assert_nan(JsNumber(NAN).as_number());
        assert_nan(JsString(String::from("a")).as_number());
    }
}
