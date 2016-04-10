use jsrs_common::types::js_var::{JsVar, JsPtrEnum, JsType};

pub type JsVarValue = (JsVar, Option<JsPtrEnum>);
pub type JsReturnValue = Option<JsVarValue>;

// Helper to avoid repeating this everywhere
pub fn scalar(v: JsType) -> (JsVar, Option<JsPtrEnum>) {
    (JsVar::new(v), None)
}

// fn push_args() { }

#[macro_export]
macro_rules! js_var_value_as_number {
    ($vv:expr) => {
        match $vv {
            (_, Some(ptr)) => ptr.as_number(),
            (var, None) => var.as_number(),
        }
    }
}
