use std::cell::RefCell;
use std::rc::Rc;

use jsrs_common::backend::Backend;
use jsrs_common::js_error;
use jsrs_common::types::coerce::AsNumber;
use jsrs_common::types::js_str::JsStrStruct;
use jsrs_common::types::js_var::{JsPtrEnum, JsType, JsVar};

use super::types::array_to_string_helper;

pub fn is_nan(state: Rc<RefCell<Backend>>, _this: Option<(JsVar, JsPtrEnum)>,
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

    Ok((JsVar::new(JsType::JsBool(number.is_nan())), None))
}

