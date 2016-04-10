mod array;

use std::cell::RefCell;
use std::rc::Rc;

use var::JsVarValue;

use french_press::ScopeManager;
use jsrs_common::backend::Backend;
use jsrs_common::types::coerce::AsString;
use jsrs_common::types::js_var::{JsType, JsPtrEnum, JsPtrTag, JsVar};
use jsrs_common::types::native_fn::NativeFn;

macro_rules! add_pervasive_fn {
    ($func:ident, $sm:expr) => {{
        let ptr_tag = JsType::JsPtr(JsPtrTag::NativeFn { name: String::from(stringify!($func)) });
        let var = JsVar::bind(stringify!($func), ptr_tag);
        let ptr = Some(JsPtrEnum::NativeFn(NativeFn::new($func)));
        (*$sm).borrow_mut().alloc(var, ptr).expect(&format!("Unable to add pervasive: {}", stringify!($func)));
    }}
}

macro_rules! add_pervasive_var {
    ($name:expr, $getter:ident, $setter:ident, $var:expr, $ptr:expr, $ts:expr, $sm:expr) => {{
        let ptr_tag = JsType::JsPtr(JsPtrTag::NativeVar { name: String::from($ts) });
        let var = JsVar::bind($name, ptr_tag);
        let ptr = Some(JsPtrEnum::NativeVar(NativeVar::new($var, $ptr, $getter, $setter)));
        (*$sm).borrow_mut().alloc(var, ptr).expect(&format!("Unable to add pervasive: {}", name));
    }}
}

pub fn add_pervasives(scope_manager: Rc<RefCell<ScopeManager>>) {
    add_pervasive_fn!(log, scope_manager);
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

pub fn default_getter(_backend: Rc<RefCell<Backend>>, var: JsVar, ptr: Option<JsPtrEnum>, _this: Option<JsPtrEnum>) -> JsVarValue {
    (var, ptr)
}
