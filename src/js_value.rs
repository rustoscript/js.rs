use std::f64::NAN;

#[derive(Debug, Clone, PartialEq)]
pub struct JsBool {
    val: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct JsError {
    val: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct JsNull;

#[derive(Debug, Clone, PartialEq)]
pub struct JsNumber {
    val: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct JsObject;

#[derive(Debug, Clone, PartialEq)]
pub struct JsString {
    val: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct JsSymbol {
    val: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct JsUndefined;

pub trait JsType {
    /// Coerce `self` into a value of JsBool for logical operations and if-statements.
    fn as_bool(self) -> JsBool;

    /// Coerce `self` into a value of JsNumber for numerical operations.
    fn as_number(self) -> JsNumber;
}

impl JsType for JsBool {
    /// Identity
    fn as_bool(self) -> JsBool {
        self
    }

    /// true -> 1; false -> 0
    fn as_number(self) -> JsNumber {
        if self.val {
            JsNumber { val: 1f64 }
        } else {
            JsNumber { val: 0f64 }
        }
    }
}

impl JsType for JsError {
    /// -> true
    fn as_bool(self) -> JsBool {
        JsBool { val: true }
    }

    /// -> NAN
    fn as_number(self) -> JsNumber {
        JsNumber { val: NAN }
    }
}

impl JsType for JsNull {
    /// -> false
    fn as_bool(self) -> JsBool {
        JsBool { val: false }
    }

    /// -> 0
    fn as_number(self) -> JsNumber {
        JsNumber { val: 0f64 }
    }
}

impl JsType for JsNumber {
    /// 0, -0, and NaN -> false; everything else -> true
    fn as_bool(self) -> JsBool {
        if self.val == 0.0f64 || self.val == -0.0f64 || self.val.is_nan() {
            JsBool { val: false }
        } else {
            JsBool { val: true }
        }
    }

    /// Identity
    fn as_number(self) -> JsNumber {
        self
    }
}

impl JsType for JsObject {
    /// -> true
    fn as_bool(self) -> JsBool {
        JsBool { val: true }
    }

    /// -> NaN
    fn as_number(self) -> JsNumber {
        JsNumber { val: NAN }
    }
}

impl JsType for JsString {
    /// empty string -> false; otherwise -> true
    fn as_bool(self) -> JsBool {
        if self.val.len() == 0 {
            JsBool { val: false }
        } else {
            JsBool { val: true }
        }
    }

    /// valid number `n` -> n; otherwise -> NaN
    fn as_number(self) -> JsNumber {
        if self.val.len() == 0 {
            JsNumber { val: 0f64 }
        } else {
            let num = self.val.parse();
            match num {
                Ok(n)  => JsNumber { val: n },
                Err(_) => JsNumber { val: NAN },
            }
        }
    }
}

impl JsType for JsSymbol {
    /// -> true
    fn as_bool(self) -> JsBool {
        JsBool { val: true }
    }

    /// TODO: actually throws error
    fn as_number(self) -> JsNumber {
        JsNumber { val: NAN }
    }
}

impl JsType for JsUndefined {
    /// -> false
    fn as_bool(self) -> JsBool {
        JsBool { val: false }
    }

    /// -> NaN
    fn as_number(self) -> JsNumber {
        JsNumber { val: NAN }
    }
}

#[cfg(test)]
mod tests {
    use std::f64::NAN;
    use super::*;

    #[test]
    fn test_as_bool() {
        assert_eq!(JsBool { val: true  }, JsBool { val: true }.as_bool());
        assert_eq!(JsBool { val: false }, JsBool { val: false }.as_bool());

        assert_eq!(JsBool { val: false }, JsUndefined.as_bool());
        assert_eq!(JsBool { val: false }, JsNull.as_bool());
        assert_eq!(JsBool { val: false }, JsNumber { val: 0.0f64 }.as_bool());
        assert_eq!(JsBool { val: false }, JsNumber { val: -0.0f64 }.as_bool());
        assert_eq!(JsBool { val: false }, JsNumber { val: NAN }.as_bool());

        assert_eq!(JsBool { val: false }, JsString { val: String::from("")  }.as_bool());
        assert_eq!(JsBool { val: true  }, JsString { val: String::from("a") }.as_bool());
        assert_eq!(JsBool { val: true  }, JsSymbol { val: String::from("")  }.as_bool());
        assert_eq!(JsBool { val: true  }, JsSymbol { val: String::from("a") }.as_bool());

        assert_eq!(JsBool { val: true  }, JsObject.as_bool());
        assert_eq!(JsBool { val: true  }, JsError { val: String::from("a") }.as_bool());
    }

    fn assert_nan(n: JsNumber) {
        assert!(n.val.is_nan());
    }

    #[test]
    fn test_as_number() {
        assert_eq!(JsNumber { val: 1f64 }, JsBool { val: true  }.as_number());
        assert_eq!(JsNumber { val: 0f64 }, JsBool { val: false }.as_number());

        assert_eq!(JsNumber { val: 0f64 }, JsNull.as_number());
        assert_eq!(JsNumber { val: 0f64 }, JsNumber { val: 0.0f64  }.as_number());
        assert_eq!(JsNumber { val: 0f64 }, JsNumber { val: -0.0f64 }.as_number());
        assert_eq!(JsNumber { val: 0f64 }, JsString { val: String::from("") }.as_number());
        assert_nan(JsSymbol { val: String::from("")  }.as_number());
        assert_nan(JsSymbol { val: String::from("a") }.as_number());
        assert_nan(JsObject.as_number());
        assert_nan(JsError { val: String::from("a") }.as_number());
        assert_nan(JsUndefined.as_number());
        assert_nan(JsNumber { val: NAN }.as_number());
        assert_nan(JsString { val: String::from("a") }.as_number());
    }
}
