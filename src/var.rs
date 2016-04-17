use jsrs_common::types::js_str::JsStrStruct;
use jsrs_common::types::js_var::{JsKey, JsPtrEnum, JsType, JsVar};

pub type JsVarValue = (JsVar, Option<JsPtrEnum>);
pub type JsReturnValue = Option<JsVarValue>;

#[inline]
// Helper to avoid repeating this everywhere
pub fn scalar(v: JsType) -> (JsVar, Option<JsPtrEnum>) {
    (JsVar::new(v), None)
}

#[inline]
pub fn js_str_key(key: &str) -> JsKey {
    JsKey::JsStr(JsStrStruct::new(key))
}

/// Loads a pointer from the scope, and returns JsError::undefined if not found.
macro_rules! try_load {
    ($state:ident, $binding:expr) => {
        match $state.borrow_mut().load($binding) {
            Ok((_, ptr)) => ptr,
            Err(_) => return Err(JsError::undefined(&$binding.0)),
        }
    }
}
