mod array;
mod log;

use std::cell::RefCell;
use std::rc::Rc;

use var::{js_str_key, JsVarValue, scalar};

use french_press::ScopeManager;
use jsrs_common::backend::Backend;
use jsrs_common::types::js_var::{JsType, JsPtrEnum, JsPtrTag, JsVar};
use jsrs_common::types::js_obj::JsObjStruct;
use jsrs_common::types::native_fn::NativeFn;
use jsrs_common::types::native_var::NativeVar;


macro_rules! add_pervasive {
    ($var:expr, $ptr:expr, $st:expr, $name:expr) => {
        $st.borrow_mut().alloc($var, $ptr).expect(&format!("Unable to add pervasive: {}", $name))
    }
}

macro_rules! add_native_fn {
    ($func:expr, $st:expr, $name:expr) => {{
        let ptr_tag = JsType::JsPtr(JsPtrTag::NativeFn { name: String::from($name) });
        let var = JsVar::bind($name, ptr_tag);
        let ptr = Some(JsPtrEnum::NativeFn(NativeFn::new($func)));
        add_pervasive!(var, ptr, $st, $name);
    }}
}

macro_rules! add_native_var {
    ($name:expr, $getter:ident, $setter:ident, $var:expr, $ptr:expr, $ts:expr, $st:expr) => {{
        let ptr_tag = JsType::JsPtr(JsPtrTag::NativeVar { name: String::from($ts) });
        let var = JsVar::bind($name, ptr_tag);
        let ptr = Some(JsPtrEnum::NativeVar(NativeVar::new($var, $ptr, $getter, $setter)));
        add_pervasive!(var, ptr, $st, $name);
    }}
}

pub fn add_pervasives(state: Rc<RefCell<ScopeManager>>) {
    add_native_fn!(log::log, state, "log");
    add_array(state)
}

pub fn get_array_proto(len: f64, state: Rc<RefCell<ScopeManager>>) -> JsObjStruct {
    let (zero, undef) = scalar(JsType::JsNum(len));
    let array_length = NativeVar::new(zero, undef, default_getter, array::array_length_setter);
    let array_push = NativeFn::new(array::array_push);

    let length_var = JsVar::new(JsType::JsPtr(JsPtrTag::NativeVar { type_string: String::from("number") }));
    let length_ptr = JsPtrEnum::NativeVar(array_length);

    let push_var = JsVar::new(JsType::JsPtr(JsPtrTag::NativeFn { name: String::from("push") }));
    let push_ptr = JsPtrEnum::NativeFn(array_push);

    let mut state_ref = state.borrow_mut();

    // Not really sure what the `name` argument is for, but okay
    let mut array_proto = JsObjStruct::new(
        None, "Array", vec![ (js_str_key("length"), length_var, Some(length_ptr))
                           , (js_str_key("push"), push_var, Some(push_ptr))
                           ], &mut *(state_ref.alloc_box.borrow_mut()));

    // No joke, the array prototype actually is an array...
    // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/prototype
    array_proto.proto = Some(Box::new(array_proto.clone()));

    array_proto
}

fn default_getter(_backend: Rc<RefCell<Backend>>, var: JsVar, ptr: Option<JsPtrEnum>, _this: Option<JsPtrEnum>) -> JsVarValue {
    (var, ptr)
}

fn add_array(state: Rc<RefCell<ScopeManager>>) {
    let array_var = JsVar::bind("Array", JsType::JsPtr(JsPtrTag::JsObj));
    let array_ptr = Some(JsPtrEnum::JsObj(get_array_proto(0.0, state.clone())));
    let mut state_ref = state.borrow_mut();
    state_ref.alloc(array_var, array_ptr).expect("Unable to alloc Array prototype");
}
