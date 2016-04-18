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
    ($state:ident, $var:expr, $is_instance_var:expr) => {{
        let mut state_ref = $state.borrow_mut();

        if $is_instance_var {
            state_ref.alloc_box.borrow_mut().find_id(&$var.unique).map(|p| p.borrow().clone())
        } else {            
            match state_ref.load(&$var.binding) {
                Ok((_, ptr)) => ptr,
                Err(_) => return Err(JsError::undefined(&$var.binding.0)),
            }
        }
    }}
}
