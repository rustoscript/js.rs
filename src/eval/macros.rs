/// eval_float_post_op!(exp, f, f - 1.0, state),
macro_rules! eval_float_post_op {
    ($e:expr, $f:ident, $new:expr, $state:expr) => {
        if let Var(ref binding) = **$e {
            let var = (*$state).borrow_mut().load(&Binding::new(binding.clone()));
            match var {
                Ok((orig_var, _)) => {
                    let $f: f64 = orig_var.as_number();
                    let new_num: f64 = $new;
                    let mut new_var = orig_var.clone();
                    new_var.t = JsNum(new_num);
                    $state.borrow_mut().store(new_var, None).unwrap();
                    Ok((orig_var, None))
                }
                _ => Err(JsError::ReferenceError(format!("ReferenceError: {} is not defined", binding)))
            }
        } else {
            Err(JsError::ReferenceError(format!("ReferenceError: invalid left-hand expression for postfix operation")))
        }
    }
}

macro_rules! eval_float_pre_op {
    ($e:expr, $f:ident, $new:expr, $state:expr) => {
        if let Var(ref binding) = **$e {
            let var = (*$state).borrow_mut().load(&Binding::new(binding.clone()));
            match var {
                Ok((orig_var, _)) => {
                    let $f: f64 = orig_var.as_number();
                    let new_num: f64 = $new;
                    let mut new_var = orig_var.clone();
                    new_var.t = JsNum(new_num);
                    $state.borrow_mut().store(new_var.clone(), None).unwrap();
                    Ok((new_var, None))
                }
                _ => Err(JsError::ReferenceError(format!("ReferenceError: {} is not defined", binding)))
            }
        } else {
            Err(JsError::ReferenceError(format!("ReferenceError: invalid left-hand expression for prefix operation")))
        }
    }
}

macro_rules! instance_var_eval {
    ($var:expr, $ptr:expr, $name:expr, $state:expr) => {
        if let JsPtr(_) = $var.t {
            match $ptr.clone() {
                Some(JsPtrEnum::JsObj(obj_struct)) => {
                    let try_inner = obj_struct.dict.get(&JsKey::JsStr(JsStrStruct::new($name)));
                    if let Some(inner_var) = try_inner {
                        let ptr = {
                            let state_ref = $state.borrow_mut();
                            let alloc_box = state_ref.alloc_box.borrow_mut();
                            alloc_box.find_id(&inner_var.unique).map(|p| {
                                p.borrow().clone()
                            })
                        };

                        match ptr.clone() {
                            Some(JsPtrEnum::NativeVar(nv)) => Ok(nv.get($state.clone(), ptr.clone())),
                            _ => Ok((inner_var.clone(), ptr)),
                        }
                    } else {
                        Ok(scalar(JsUndef))
                    }
                },
                // TODO: all JsPtrs can have instance vars/methods, not just JsObjs
                _ => Err(JsError::UnimplementedError(String::from("InstanceVar, eval/mod.rs:295")))
            }
        } else {
            // TODO: Things which are not ptrs can also have instance vars/methods
            Err(JsError::UnimplementedError(String::from("InstanceVar, eval/mod.rs:299")))
        }
    }
}

macro_rules! instance_var_assign {
    ($e:expr, $state:expr, $rhs_var:expr, $rhs_ptr:expr, $string:expr) => {{
        let (var, ptr) = try!(eval_exp(&$e.clone(), $state.clone()));

        let mut obj = match ptr.clone() {
            Some(JsPtrEnum::JsObj(obj)) => obj,
            _ => return Ok((($rhs_var, $rhs_ptr), None))
        };


        let native_var = match obj.dict.get(&js_str_key($string)) {
            Some(ref var) => {
                let state_ref = $state.borrow_mut();
                let alloc_box = state_ref.alloc_box.borrow_mut();
                alloc_box.find_id(&var.unique).map(|p| p.borrow().clone())
            }
            None => None
        };


        if let Some(JsPtrEnum::NativeVar(mut nv)) = native_var {
            nv.set($state.clone(), ptr.clone()
            .map(|x| (var.clone(), x)), $rhs_var, $rhs_ptr);
            return Ok(((nv.var.clone(), nv.clone().ptr.map(|x| *x)), None));
        }

        $rhs_var.mangle($string);

        let mut state_ref = $state.borrow_mut();
        obj.add_key(&var.unique,
            JsKey::JsStr(JsStrStruct::new($string)),
            $rhs_var.clone(), $rhs_ptr.clone(),
            &mut *(state_ref.alloc_box.borrow_mut()));
        state_ref.store(var, Some(JsPtrEnum::JsObj(obj))).expect("Unable to store changed object");
        $rhs_var
    }}
}
