use std::fmt;
use std::result;

use jsrs_common::gc_error::GcError;

#[derive(Debug)]
pub enum JsError {
    ParseError(String),
    GcError(GcError),
    TypeError(String),
    ReferenceError(String),
}

impl fmt::Display for JsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            JsError::ParseError(ref s) => write!(f, "ParseError: {}", s),
            JsError::GcError(ref gc) => write!(f, "GcError: {}", gc),
            JsError::TypeError(ref s) => write!(f, "TypeError: {}", s),
            JsError::ReferenceError(ref s) => write!(f, "ReferenceError: {}", s),
        }
    }
}

pub type Result<T> = result::Result<T, JsError>;
