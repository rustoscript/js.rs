/// eval_float_post_op!(exp, f, f - 1.0, state),
macro_rules! eval_float_post_op {
    ($e:expr, $f:ident, $new:expr, $state:expr) => {
        if let Var(ref binding) = **$e {
            let var = $state.deref().borrow().load(&Binding::new(binding.clone()));
            match var {
                Ok((orig_var, _)) => {
                    let $f: f64 = orig_var.as_number();
                    let new_num: f64 = $new;
                    let new_var = JsVar::bind(&binding, JsNum(new_num));
                    $state.deref().borrow_mut().alloc(new_var, None).unwrap();
                    orig_var
                }
                _ => panic!(format!("ReferenceError: {} is not defined", binding))
            }
        } else {
            panic!("invalid left-hand expression for postfix operation");
        }
    }
}

macro_rules! eval_float_pre_op {
    ($e:expr, $f:ident, $new:expr, $state:expr) => {
        if let Var(ref binding) = **$e {
            let var = $state.deref().borrow().load(&Binding::new(binding.clone()));
            match var {
                Ok((orig_var, _)) => {
                    let $f: f64 = orig_var.as_number();
                    let new_num: f64 = $new;
                    let new_var = JsVar::bind(&binding, JsNum(new_num));
                    $state.deref().borrow_mut().alloc(new_var.clone(), None).unwrap();
                    new_var
                }
                _ => panic!(format!("ReferenceError: {} is not defined", binding))
            }
        } else {
            panic!("invalid left-hand expression for postfix operation");
        }
    }
}
