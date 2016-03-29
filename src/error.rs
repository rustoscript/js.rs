use std::fmt;
use std::result;

use french_press::gc_error::GcError;


#[derive(Debug)]
pub enum JsError {
    ParseError(String),
    GcError(GcError),
}

impl fmt::Display for JsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            JsError::ParseError(ref s) => write!(f, "ParseError: {}", s),
            JsError::GcError(ref gc) => write!(f, "GcError: {}", gc),
        }
    }
}

pub type Result<T> = result::Result<T, JsError>;
