use std::cell::RefCell;
use std::rc::Rc;

use jsrs_common::backend::Backend;
use jsrs_common::js_error::{self, JsError};
use jsrs_common::types::coerce::AsString;
use jsrs_common::types::js_var::{JsType, JsPtrEnum, JsVar};

use native::types::string;

pub fn log(state: Rc<RefCell<Backend>>, this: Option<(JsVar, JsPtrEnum)>,
       args: Vec<(JsVar, Option<JsPtrEnum>)>) -> js_error::Result<(JsVar, Option<JsPtrEnum>)> {
    let (var, ptr) = try!(string(state.clone(), this, args));
    println!("{}", ptr.map(|p| p.as_string()).unwrap_or(var.t.as_string()));
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
