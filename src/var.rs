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

// fn push_args() { }
