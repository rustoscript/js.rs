use std::cell::RefCell;
use std::rc::Rc;

use jsrs_common::backend::Backend;
use jsrs_common::types::coerce::AsString;
use jsrs_common::types::js_var::{JsType, JsPtrEnum, JsVar};

use jsrs_common::js_error::{self, JsError};

pub fn log(_scope: Rc<RefCell<Backend>>, _this: Option<(JsVar, JsPtrEnum)>,
       args: Vec<(JsVar, Option<JsPtrEnum>)>) -> js_error::Result<(JsVar, Option<JsPtrEnum>)> {
    match args.first() {
        Some(&(_, Some(ref var))) => println!("{}", var.as_string()),
        Some(&(ref var, _)) => println!("{}", var.t.as_string()),
        None => println!("")
    };

    Ok((JsVar::new(JsType::JsNull), None))
}

pub fn error(_scope: Rc<RefCell<Backend>>,
             _this: Option<(JsVar, JsPtrEnum)>,
            args: Vec<(JsVar, Option<JsPtrEnum>)>) -> js_error::Result<(JsVar, Option<JsPtrEnum>)> {
    let s = match args.first() {
        Some(&(_, Some(ref var))) => var.as_string(),
        Some(&(ref var, _)) => var.t.as_string(),
        None => String::from(""),
    };

    println!("{}", s);

    //let (var, ptr) = try!(eval_exp(exp, state));
    Err(JsError::TestError(s))
    //(JsVar::new(JsType::JsNull), None)
}
