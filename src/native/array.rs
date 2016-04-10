use std::cell::RefCell;
use std::rc::Rc;

use var::JsVarValue;

use jsrs_common::backend::Backend;
use jsrs_common::types::coerce::{AsNumber, AsString};
use jsrs_common::types::js_str::JsStrStruct;
use jsrs_common::types::js_var::{JsKey, JsPtrEnum, JsType, JsVar};
use jsrs_common::types::native_var::NativeVar;

pub fn array_length_setter(state: Rc<RefCell<Backend>>, old_var: JsVar, old_ptr: Option<JsPtrEnum>, this: Option<JsPtrEnum>,
               new_var: JsVar, new_ptr: Option<JsPtrEnum>) -> JsVarValue {
    let new_len = js_var_value_as_number!((new_var, new_ptr));

    if !(new_len.is_normal() || new_len == 0.0) || new_len.is_sign_negative() || new_len != new_len.trunc() {
        // TODO: Return a `RangeError` instead of panicking.
        panic!("Invalid array length:\n var: {:#?}\nptr: {:#?}", new_var, new_ptr);
    }

    let old_len = js_var_value_as_number!((old_var, old_ptr));

    let mut this_obj = match this {
        Some(JsPtrEnum::JsObj(obj)) => obj,
        Some(_) => panic!("Trying to set array length, but `this` is not an object"),
        None => panic!("Trying to set array length, but `this` is None")
    };

    let new_len_int = new_len as i32;
    let old_len_int = old_len as i32;

    if new_len > old_len {
        for i in old_len_int..new_len_int {
            let key = JsKey::JsStr(JsStrStruct::new(&JsType::JsNum(i as f64).as_string()));
            this_obj.add_key(key, JsVar::new(JsType::JsUndef), None, state.borrow_mut().get_allocator().borrow_mut());
        }
    }

    if new_len < old_len {
        for i in new_len_int..old_len_int {
            let key = JsKey::JsStr(JsStrStruct::new(&JsType::JsNum(i as f64).as_string()));
            let _ = this_obj.dict.remove(&key);
        }
    }

    (JsVar::new(JsType::JsNum(new_len)), None)
}
