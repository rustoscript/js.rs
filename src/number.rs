use std::cell::RefCell;
use std::rc::Rc;

use french_press::ScopeManager;
use jsrs_common::backend::Backend;

use jsrs_common::ast::*;
use jsrs_common::ast::BinOp::*;
use jsrs_common::types::coerce::{AsBool, AsNumber, AsString};
use jsrs_common::types::js_var::{JsVar, JsType};
use jsrs_common::types::js_var::JsPtrEnum::*;
use jsrs_common::types::js_var::JsType::*;
use jsrs_common::types::js_var::{JsPtrEnum, JsPtrTag};
use jsrs_common::types::js_str::JsStrStruct;
use jsrs_common::js_error::{self, JsError};

use var::{JsVarValue, scalar};
use eval::eval_exp;

macro_rules! b { ($e: expr) => { $e.as_bool() } }

macro_rules! n { ($e: expr) => { $e.as_number() } }
macro_rules! ni64 { ($e: expr) => { $e.as_number() as i64 } }
macro_rules! nu64 { ($e: expr) => { $e.as_number() as u64 } }
macro_rules! ni32 { ($e: expr) => { {
        let n = $e.as_number();
        if n.is_nan() {
            0i32
        } else {
            n as i32
        }
    } } }
macro_rules! nu32 { ($e: expr) => { $e.as_number() as u32 } }

pub fn eval_binop(op: &BinOp, e1: &Exp, e2: &Exp,
                  state: Rc<RefCell<ScopeManager>>) -> js_error::Result<JsVarValue> {
    if let &And = op {
        let val1: JsVar = try!(eval_exp(e1, state.clone())).0;
        let b = if b!(val1) == false {
            JsBool(false)
        } else {
            let val2: JsVar = try!(eval_exp(e2, state.clone())).0;
            JsBool(b!(val2))
        };
        return Ok(scalar(b));
    } else if let &Or = op {
        let val1: JsVar = try!(eval_exp(e1, state.clone())).0;
        let b = if b!(val1) == true {
            JsBool(true)
        } else {
            let val2: JsVar = try!(eval_exp(e2, state.clone())).0;
            JsBool(b!(val2))
        };
        return Ok(scalar(b));
    }

    let val1_is_instance_var = match e1 {
        &Exp::InstanceVar(..) |
        &Exp::KeyAccessor(..) => true,
        _ => false
    };

    let val2_is_instance_var = match e2 {
        &Exp::InstanceVar(..) |
        &Exp::KeyAccessor(..) => true,
        _ => false
    };

    let (val1, ptr1) = try!(eval_exp(e1, state.clone()));
    let (val2, ptr2) = try!(eval_exp(e2, state.clone()));

    if let Err(e) = state.borrow_mut().alloc(val1.clone(), ptr1.clone()) {
        return Err(JsError::from(e));
    }

    if let Err(e) = state.borrow_mut().alloc(val2.clone(), ptr2.clone()) {
        return Err(JsError::from(e));
    }

    let v = match *op {
        And => {
            println!("{:?}", val1);
            if b!(val1) == false {
                JsBool(false)
            } else {
                JsBool(b!(val2))
            }
        }
        Or  => JsBool(b!(val1) || b!(val2)),

        Ge  => JsBool(n!(val1) >= n!(val2)),
        Gt  => JsBool(n!(val1) >  n!(val2)),
        Le  => JsBool(n!(val1) <= n!(val2)),
        Lt  => JsBool(n!(val1) <  n!(val2)),

        Neq => {
            if let Ok(JsBool(b)) = eval_binop(&Eql, e1, e2, state).map(|(x, _)| x.t) {
                JsBool(!b)
            } else {
                JsBool(false)
            }
        }
        Eql => {
            let b = match (&val1.t, &val2.t) {
                (&JsNull,  &JsNull)  => false,
                (&JsUndef, &JsNull)  => false,
                (&JsNull,  &JsUndef) => false,
                (&JsUndef, &JsUndef) => false,

                (&JsNum(ref n1), &JsNum(ref n2)) => n1 == n2,
                (&JsBool(ref b1), &JsBool(ref b2)) => b1 == b2,
                (&JsPtr(_), &JsPtr(_)) => {
                    let ptr1 = try_load!(state, &val1, val1_is_instance_var);
                    let ptr2 = try_load!(state, &val2, val2_is_instance_var);
                    match (&ptr1, &ptr2) {
                        (&Some(JsSym(_)),      &Some(JsSym(_))) => val1 == val2,
                        (&Some(JsStr(ref s1)), &Some(JsStr(ref s2))) => s1 == s2,
                        (&Some(JsObj(_)),      &Some(JsObj(_))) => val1 == val2,
                        (&Some(JsFn(_)),       &Some(JsFn(_))) => val1 == val2,
                        _ => false,
                    }
                },

                (&JsNum(ref n), &JsPtr(_)) =>
                    try_load!(state, &val2, val2_is_instance_var).map_or(false, |ptr| *n == n!(ptr)),
                (&JsPtr(_), &JsNum(ref n)) =>
                    try_load!(state, &val2,val2_is_instance_var).map_or(false, |ptr| *n == n!(ptr)),

                (&JsBool(_), &JsPtr(_)) =>
                    try_load!(state, &val2, val2_is_instance_var).map_or(false, |ptr| n!(val1) == n!(ptr)),
                (&JsPtr(_), &JsBool(_)) =>
                    try_load!(state, &val2, val2_is_instance_var).map_or(false, |ptr| n!(val2) == n!(ptr)),
                _ => false,
            };
            JsBool(b)
        }

        EqlStrict => {
            let b = match (&val1.t, &val2.t) {
                (&JsNull,      &JsNull) => true,
                (&JsUndef,     &JsUndef) => true,
                (&JsNum(ref n1),   &JsNum(ref n2)) => n1 == n2,
                (&JsBool(ref b1),  &JsBool(ref b2)) => b1 == b2,
                (&JsPtr(_), &JsPtr(_)) => {
                    let ptr1 = try_load!(state, &val1, val1_is_instance_var);
                    let ptr2 = try_load!(state, &val2, val2_is_instance_var);
                    match (&ptr1, &ptr2) {
                        (&Some(JsSym(_)),      &Some(JsSym(_))) => val1 == val2,
                        (&Some(JsStr(ref s1)), &Some(JsStr(ref s2))) => s1 == s2,
                        (&Some(JsObj(_)),      &Some(JsObj(_))) => val1 == val2,
                        (&Some(JsFn(_)),       &Some(JsFn(_))) => val1 == val2,
                        _ => false,
                    }
                }
                _ => false,
            };
            JsBool(b)
        }
        NeqStrict => {
            if let Ok(JsBool(b)) = eval_binop(&EqlStrict, e1, e2, state).map(|(x, _)| x.t) {
                JsBool(!b)
            } else {
                JsBool(false)
            }
        }

        BitOr => JsNum((ni32!(val1) | ni32!(val2)) as f64),
        BitXor => JsNum((ni32!(val1) ^ ni32!(val2)) as f64),
        BitAnd => JsNum((ni32!(val1) & ni32!(val2)) as f64),

        // TODO: Rust panics on shift overflow, and I don't want this.
        ShiftLeft          => JsNum(0.0),
        ShiftRight         => JsNum(0.0),
        ShiftRightUnsigned => JsNum(0.0),
        //ShiftLeft          => JsNum((ni32!(val1) << ni32!(val2)) as f64),
        //ShiftRight         => JsNum((ni32!(val1) >> ni32!(val2)) as f64),
        //ShiftRightUnsigned => JsNum((nu32!(val1) >> ni32!(val2)) as f64),

        Minus => JsNum(n!(val1) - n!(val2)),
        Plus  => {
            if let JsPtr(JsPtrTag::JsStr) = val1.t {
                let mut s1 = ptr1.map(|p| p.as_string()).unwrap_or(val1.t.as_string());
                let s2 = ptr2.map(|p| p.as_string()).unwrap_or(val2.t.as_string());
                s1.push_str(&s2);

                let var = JsVar::new(JsType::JsPtr(JsPtrTag::JsStr));
                let ptr = JsPtrEnum::JsStr(JsStrStruct::new(&s1));
                return Ok((var, Some(ptr)));
            }

            JsNum(n!(val1) + n!(val2))
        }
        Slash => JsNum(n!(val1) / n!(val2)),
        Star  => JsNum(n!(val1) * n!(val2)),
        Mod   => JsNum(n!(val1) % n!(val2)),
        Exponent   => JsNum(n!(val1) % n!(val2)),
        InstanceOf => {
            let ptr = try_load!(state, &val1, val1_is_instance_var);

            let b = match (ptr, &val2.t) {
                (Some(JsObj(ref obj)), &JsPtr(JsPtrTag::NativeFn { ref name})) => &obj.name == name,
                (_, &JsPtr(JsPtrTag::NativeFn {..})) => false,
                (_, &JsPtr(JsPtrTag::JsFn{..})) => false,
                _ => {
                    let ptr2 = try_load!(state, &val2, val2_is_instance_var);
                    let err_str = ptr2.map(|p| p.as_string()).unwrap_or(val2.t.as_string());
                    return Err(JsError::TypeError(format!("Expecting a function in instanceof check, but got {}", err_str)));
                }
            };

            JsBool(b)
        }
    };
    Ok(scalar(v))
}
