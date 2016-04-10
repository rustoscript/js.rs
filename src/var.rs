use jsrs_common::types::js_var::{JsVar, JsPtrEnum, JsType};

pub type JsVarValue = (JsVar, Option<JsPtrEnum>);
pub type JsReturnValue = Option<JsVarValue>;

// Helper to avoid repeating this everywhere
pub fn scalar(v: JsType) -> (JsVar, Option<JsPtrEnum>) {
    (JsVar::new(v), None)
}

// fn push_args() { }
