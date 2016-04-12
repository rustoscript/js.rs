use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

use french_press::ScopeManager;
use jsrs_common::backend::Backend;
use jsrs_common::types::native_fn::NativeFn;
use jsrs_common::types::coerce::{AsString, AsNumber};
use jsrs_common::types::js_var::{JsType, JsPtrEnum, JsPtrTag, JsVar};
use var::scalar;


macro_rules! add_pervasive {
    ($func:ident, $sm:expr) => {{
        let ptr_tag = JsType::JsPtr(JsPtrTag::NativeFn { name: String::from(stringify!($func)) });
        let var = JsVar::bind(stringify!($func), ptr_tag);
        let ptr = Some(JsPtrEnum::NativeFn(NativeFn::new($func)));
        $sm.deref().borrow_mut().alloc(var, ptr)
            .expect(&format!("Unable to add pervasive: {}", stringify!($func))[..]);
    }}
}

macro_rules! add_named_pervasive {
    ($func:ident, $name: expr, $sm:expr) => {{
        let name = String::from($name);
        let ptr_tag = JsType::JsPtr(JsPtrTag::NativeFn { name: name });
        let var = JsVar::bind($name, ptr_tag);
        let ptr = Some(JsPtrEnum::NativeFn(NativeFn::new($func)));
        $sm.deref().borrow_mut().alloc(var, ptr)
            .expect(&format!("Unable to add pervasive: {}", stringify!($func))[..]);
    }}
}

pub fn add_pervasives(scope_manager: Rc<RefCell<ScopeManager>>) {
    add_pervasive!(log, scope_manager);
    add_pervasive!(isNaN, scope_manager);
    add_named_pervasive!(error, "$ERROR", scope_manager);
}

fn log(_scope: Rc<RefCell<Backend>>, _this: Option<JsPtrEnum>,
       args: Vec<(JsVar, Option<JsPtrEnum>)>) -> (JsVar, Option<JsPtrEnum>) {
    match args.first() {
        Some(&(_, Some(ref var))) => println!("{}", var.as_string()),
        Some(&(ref var, _)) => println!("{}", var.t.as_string()),
        None => println!("")
    };

    (JsVar::new(JsType::JsNull), None)
}

#[allow(non_snake_case)]
fn isNaN(_scope: Rc<RefCell<Backend>>, _this: Option<JsPtrEnum>,
       args: Vec<(JsVar, Option<JsPtrEnum>)>) -> (JsVar, Option<JsPtrEnum>) {
    if let Some(&(ref var, _)) = args.first() {
        scalar(JsType::JsBool(var.as_number().is_nan()))
    } else {
        scalar(JsType::JsBool(true))
    }
}

fn error(_scope: Rc<RefCell<Backend>>, _this: Option<JsPtrEnum>,
       args: Vec<(JsVar, Option<JsPtrEnum>)>) -> (JsVar, Option<JsPtrEnum>) {
    print!("$ERROR: ");
    match args.first() {
        Some(&(_, Some(ref var))) => println!("{}", var.as_string()),
        Some(&(ref var, _)) => println!("{}", var.t.as_string()),
        None => println!("")
    };

    (JsVar::new(JsType::JsNull), None)
}
