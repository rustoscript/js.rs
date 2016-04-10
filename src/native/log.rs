use std::cell::RefCell;
use std::rc::Rc;

use jsrs_common::backend::Backend;
use jsrs_common::types::coerce::AsString;
use jsrs_common::types::js_var::{JsType, JsPtrEnum, JsVar};

pub fn log(_scope: Rc<RefCell<Backend>>, _this: Option<JsPtrEnum>,
       args: Vec<(JsVar, Option<JsPtrEnum>)>) -> (JsVar, Option<JsPtrEnum>) {
    match args.first() {
        Some(&(_, Some(ref var))) => println!("{}", var.as_string()),
        Some(&(ref var, _)) => println!("{}", var.t.as_string()),
        None => println!("")
    };

    (JsVar::new(JsType::JsNull), None)
}
