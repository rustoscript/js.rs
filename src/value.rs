use std::fmt::{Display, Error, Formatter};

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum JsValue {
    // Primitives (cannot be changed)
    JsBoolean(bool),
    JsUndefined,
    JsNull,
    JsNumber(f64),
    JsString(String),
    JsSymbol(String),
    // Objects
    // TODO: internal representation
    JsObject,
    // Error value (TODO: more consistent to ECMAScript spec?)
    JsError(String),
}

use self::JsValue::*;

impl Display for JsValue {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match *self {
            JsBoolean(b) => match b {
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::JsValue::*;
    use std::f64::{NAN, INFINITY, NEG_INFINITY};

    #[test]
    fn jsvalue_display() {
        assert_eq!("true", format!("{}", JsBoolean(true)));
        assert_eq!("false", format!("{}", JsBoolean(false)));
        assert_eq!("undefined", format!("{}", JsUndefined));
        assert_eq!("null", format!("{}", JsNull));

        assert_eq!("4", format!("{}", JsNumber(4f64)));
        assert_eq!("-4", format!("{}", JsNumber(-4f64)));
        assert_eq!("0", format!("{}", JsNumber(0f64)));
        assert_eq!("NaN", format!("{}", JsNumber(NAN)));
        assert_eq!("inf", format!("{}", JsNumber(INFINITY)));
        assert_eq!("-inf", format!("{}", JsNumber(NEG_INFINITY)));

        assert_eq!("", format!("{}", JsString(String::from(""))));
        assert_eq!("string", format!("{}", JsString(String::from("string"))));

        assert_eq!("", format!("{}", JsSymbol(String::from(""))));
        assert_eq!("string", format!("{}", JsSymbol(String::from("string"))));

        assert_eq!("{}", format!("{}", JsObject));
    }
}
