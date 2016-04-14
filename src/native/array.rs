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

pub fn array_push(state: Rc<RefCell<Backend>>, this: Option<(JsVar, JsPtrEnum)>, args: Vec<(JsVar, Option<JsPtrEnum>)>) -> (JsVar, Option<JsPtrEnum>) {
    // TODO: Change panics to actual errors
    let (this_var, mut this_obj) = match this.clone() {
        Some((v, JsPtrEnum::JsObj(obj))) => (v, obj),
        Some(_) => panic!("Trying to push onto array, but `this` is not an object"),
        None => panic!("Trying to push onto array, but `this` is None")
    };

    let mut length_ptr = match this_obj.dict.get(&js_str_key("length")) {
        Some(js_var) => {
            let state_ref = state.borrow_mut();
            let alloc_box = state_ref.get_alloc_box();
            let alloc_ref = alloc_box.borrow_mut();
            match alloc_ref.find_id(&js_var.unique).map(|p| p.borrow().clone()) {
                Some(JsPtrEnum::NativeVar(native_var)) => native_var,
                Some(_) => panic!("Array length pointer is not a native variable"),
                None => panic!("No pointer for array length"),
            }
        }
        None => panic!("No length field on array"),
    };

    let length = match length_ptr.get(state.clone(), this.clone().map(|x| x.1)).0.t {
        JsType::JsNum(f) => f,
        _ => panic!("Array length value is not a number"),
    };

    let new_length = JsVar::new(JsType::JsNum(length + args.len() as f64));
    length_ptr.set(state.clone(), this, new_length.clone(), None);
    let state_ref = state.borrow_mut();
    let alloc_box = state_ref.get_alloc_box();

    for (i, (var, ptr)) in args.into_iter().enumerate() {
        let key = JsKey::JsStr(JsStrStruct::new(&JsType::JsNum(length + i as f64).as_string()));
        this_obj.add_key(&this_var.unique, key, var, ptr, &mut *(alloc_box.borrow_mut()));
    }

    (new_length, None)
}

pub fn array_length_setter(state: Rc<RefCell<Backend>>, old_var: JsVar, old_ptr: Option<JsPtrEnum>,
                           this: Option<(JsVar, JsPtrEnum)>,  new_var: JsVar, new_ptr: Option<JsPtrEnum>) -> JsVarValue {
    let new_len = var_type_as_number(&new_var, new_ptr.as_ref());
    let old_len = var_type_as_number(&old_var, old_ptr.as_ref());

    if !(new_len.is_normal() || new_len == 0.0) || new_len.is_sign_negative() || new_len != new_len.trunc() {
        // TODO: Return a `RangeError` instead of panicking.
        panic!("Invalid array length:\n var: {:#?}\nptr: {:#?}", new_var, new_ptr);
    }

    let (this_var, mut this_obj) = match this.clone() {
        Some((v, JsPtrEnum::JsObj(obj))) => (v, obj),
        Some(_) => panic!("Trying to set array length, but `this` is not an object"),
        None => panic!("Trying to set array length, but `this` is None")
    };

    let new_len_int = new_len as i32;
    let old_len_int = old_len as i32;

    let state_ref = state.borrow_mut();

    if new_len > old_len {
        for i in old_len_int..new_len_int {
            let key = JsKey::JsStr(JsStrStruct::new(&JsType::JsNum(i as f64).as_string()));
            let alloc_box = state_ref.get_alloc_box();
            this_obj.add_key(&this_var.unique, key, JsVar::new(JsType::JsUndef), None, &mut *(alloc_box.borrow_mut()));
        }
    }

    if new_len < old_len {
        for i in new_len_int..old_len_int {
            let key = js_str_key(&JsType::JsNum(i as f64).as_string());
            let alloc_box = state_ref.get_alloc_box();
            this_obj.remove_key(&this_var.unique, &key, &mut *(alloc_box.borrow_mut()));
        }
    }

    (JsVar::new(JsType::JsNum(new_len)), None)
}
