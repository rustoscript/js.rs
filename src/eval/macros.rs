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
