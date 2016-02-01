/// eval_float_post_op!(exp, f, f - 1.0, state),
macro_rules! eval_float_post_op {
    ($e:expr, $f:ident, $new:expr, $state:expr) => {
        if let Var(ref binding) = **$e {
            match $state.load(&Binding::new(binding)) {
                Ok((orig_var, _)) => {
                        let $f: f64 = orig_var.as_number();
                        let new_num: f64 = $new;
                        let mut new_var = JsVar::new(JsNum(new_num));
                        new_var.binding = Binding::new(&binding);
                        $state.alloc(new_var, None).unwrap();
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
            match $state.load(&Binding::new(binding)) {
                Ok((orig_var, _)) => {
                        let $f: f64 = orig_var.as_number();
                        let new_num: f64 = $new;
                        let mut new_var = JsVar::new(JsNum(new_num));
                        new_var.binding = Binding::new(&binding);
                        $state.alloc(new_var.clone(), None).unwrap();
                        new_var
                }
                _ => panic!(format!("ReferenceError: {} is not defined", binding))
            }
        } else {
            panic!("invalid left-hand expression for postfix operation");
        }
    }
}
