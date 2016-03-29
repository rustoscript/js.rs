//use french_press::ScopeManager;
//use french_press::alloc::AllocBox;
//use js_types::binding::Binding;
//use js_types::js_fn::JsFnStruct;
//use js_types::js_obj::JsObjStruct;
//use js_types::js_str::JsStrStruct;
//use js_types::js_var::{JsVar, JsType, JsPtrEnum, JsKey, JsPtrTag};
//use js_types::js_var::JsType::*;
use js_types::js_var::{JsVar, JsPtrEnum, JsType};


pub type JsVarValue = (JsVar, Option<JsPtrEnum>);

pub type JsReturnValue = Option<JsVar>;

// Helper to avoid repeating this everywhere
pub fn scalar(v: JsType) -> (JsVar, Option<JsPtrEnum>) {
    (JsVar::new(v), None)
}

fn push_args() {
}
