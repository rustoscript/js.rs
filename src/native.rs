use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

use french_press::ScopeManager;
use jsrs_common::backend::Backend;
use jsrs_common::types::native_fn::NativeFn;
use jsrs_common::types::coerce::AsString;
use jsrs_common::types::js_var::{JsType, JsPtrEnum, JsPtrTag, JsVar};

pub fn add_pervasives(scope_manager: Rc<RefCell<ScopeManager>>) {
    scope_manager.deref().borrow_mut().alloc(JsVar::bind("log", JsType::JsPtr(JsPtrTag::NativeFn { name: String::from("log") })),
                        Some(JsPtrEnum::NativeFn(NativeFn::new(log))));
}

fn log(scope: Rc<RefCell<Backend>>, args: Vec<(JsVar, Option<JsPtrEnum>)>) -> (JsVar, Option<JsPtrEnum>) {
    match args.first() {
        Some(&(_, Some(ref var))) => println!("{}", var.as_string()),
        Some(&(ref var, _)) => println!("{}", var.t.as_string()),
        None => println!("")
    };

    (JsVar::new(JsType::JsNull), None)
}
