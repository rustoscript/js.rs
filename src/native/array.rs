use std::cell::RefCell;
use std::rc::Rc;

use var::{js_str_key, JsVarValue};

use jsrs_common::backend::Backend;
use jsrs_common::types::coerce::{AsNumber, AsString};
use jsrs_common::types::js_str::JsStrStruct;
use jsrs_common::types::js_var::{JsKey, JsPtrEnum, JsType, JsVar};

fn var_type_as_number(var: &JsVar, ptr: Option<&JsPtrEnum>) -> f64 {
    match ptr {
        Some(ref ptr) => ptr.as_number(),
        None => var.as_number()
    }
}

pub fn array_length_setter(state: Rc<RefCell<Backend>>, old_var: JsVar, old_ptr: Option<JsPtrEnum>, this: Option<JsPtrEnum>,
               new_var: JsVar, new_ptr: Option<JsPtrEnum>) -> JsVarValue {
    let new_len = var_type_as_number(&new_var, new_ptr.as_ref());
    let old_len = var_type_as_number(&old_var, old_ptr.as_ref());

    if !(new_len.is_normal() || new_len == 0.0) || new_len.is_sign_negative() || new_len != new_len.trunc() {
        // TODO: Return a `RangeError` instead of panicking.
        panic!("Invalid array length:\n var: {:#?}\nptr: {:#?}", new_var, new_ptr);
    }

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
            let alloc_box = state.borrow_mut().get_alloc_box();
            this_obj.add_key(key, JsVar::new(JsType::JsUndef), None, &mut *(alloc_box.borrow_mut()));
        }
    }

    if new_len < old_len {
        for i in new_len_int..old_len_int {
            let key = js_str_key(&JsType::JsNum(i as f64).as_string());
            let _ = this_obj.dict.remove(&key);
        }
    }

    (JsVar::new(JsType::JsNum(new_len)), None)
}
