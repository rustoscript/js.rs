use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

use french_press::ScopeManager;
use jsrs_common::backend::Backend;
use jsrs_common::types::native_fn::NativeFn;
use jsrs_common::types::coerce::AsString;
use jsrs_common::types::js_var::{JsType, JsPtrEnum, JsPtrTag, JsVar};

macro_rules! add_pervasive {
    ($func:ident, $sm:expr) => {{
        let ptr_tag = JsType::JsPtr(JsPtrTag::NativeFn { name: String::from(stringify!($func)) });
        let var = JsVar::bind(stringify!($func), ptr_tag);
        let ptr = Some(JsPtrEnum::NativeFn(NativeFn::new($func)));
        $sm.deref().borrow_mut().alloc(var, ptr).expect(&format!("Unable to add pervasive: {}", stringify!($func))[..]);
    }}
}

pub fn add_pervasives(scope_manager: Rc<RefCell<ScopeManager>>) {
    add_pervasive!(log, scope_manager);
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
