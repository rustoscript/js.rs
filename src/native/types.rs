use std::cell::RefCell;
use std::rc::Rc;

use jsrs_common::backend::Backend;
use jsrs_common::js_error;
use jsrs_common::types::coerce::{AsBool, AsNumber, AsString};
use jsrs_common::types::js_obj::JsObjStruct;
use jsrs_common::types::js_str::JsStrStruct;
use jsrs_common::types::js_var::{JsPtrEnum, JsPtrTag, JsType, JsVar};

use super::array::array_to_string;

pub fn object(state: Rc<RefCell<Backend>>, _this: Option<(JsVar, JsPtrEnum)>,
              args: Vec<(JsVar, Option<JsPtrEnum>)>) -> js_error::Result<(JsVar, Option<JsPtrEnum>)> {
    let state_ref = state.borrow_mut();
    let alloc_box = state_ref.get_alloc_box();
    let (var, ptr) = args.first().map(|&(ref var, ref ptr)| (var.clone(), ptr.clone())).unwrap_or((
        JsVar::new(JsType::JsPtr(JsPtrTag::JsObj)),
        Some(JsPtrEnum::JsObj(JsObjStruct::new(None, "Object", Vec::new(), &mut *(alloc_box.borrow_mut()))))
    ));

    Ok((var, ptr))
}

pub fn boolean(_state: Rc<RefCell<Backend>>, _this: Option<(JsVar, JsPtrEnum)>,
       args: Vec<(JsVar, Option<JsPtrEnum>)>) -> js_error::Result<(JsVar, Option<JsPtrEnum>)> {
    let boolean = args.first().map(|&(ref var, ref ptr)| ptr.as_ref().map(|p| p.as_bool()).unwrap_or(var.as_bool()));
    Ok((JsVar::new(JsType::JsBool(boolean.unwrap_or(false))), None))
}

pub fn number(state: Rc<RefCell<Backend>>, _this: Option<(JsVar, JsPtrEnum)>,
       args: Vec<(JsVar, Option<JsPtrEnum>)>) -> js_error::Result<(JsVar, Option<JsPtrEnum>)> {
    let number = match args.first() {
        Some(&(ref var, Some(JsPtrEnum::JsObj(ref obj)))) if obj.proto.is_some() && obj.name == "array" => {
            let string = try!(array_to_string_helper(state.clone(), var.clone(), obj.clone()));
            JsPtrEnum::JsStr(JsStrStruct::new(&string)).as_number()
        }
        Some(&(_, Some(ref ptr))) => ptr.as_number(),
        Some(&(ref var, None)) => var.as_number(),
        None => 0.0
    };

    Ok((JsVar::new(JsType::JsNum(number)), None))
}

pub fn array_to_string_helper(state: Rc<RefCell<Backend>>, var: JsVar, obj: JsObjStruct) -> js_error::Result<String> {
    let o_this = Some((var, JsPtrEnum::JsObj(obj)));
    let (o_var, o_ptr) = try!(array_to_string(state.clone(), o_this, Vec::new()));
    Ok(o_ptr.map(|p| p.as_string()).unwrap_or(o_var.t.as_string()))
}

pub fn string(state: Rc<RefCell<Backend>>, _this: Option<(JsVar, JsPtrEnum)>,
       args: Vec<(JsVar, Option<JsPtrEnum>)>) -> js_error::Result<(JsVar, Option<JsPtrEnum>)> {
    let string = match args.first() {
        Some(&(ref var, Some(JsPtrEnum::JsObj(ref obj)))) if obj.proto.is_some() && obj.name == "array" =>
            try!(array_to_string_helper(state.clone(), var.clone(), obj.clone())),
        Some(&(_, Some(ref ptr))) => ptr.as_string(),
        Some(&(ref var, None)) => var.t.as_string(),
        None => String::new()
    };


    Ok((JsVar::new(JsType::JsPtr(JsPtrTag::JsStr)), Some(JsPtrEnum::JsStr(JsStrStruct::new(&string)))))
}
