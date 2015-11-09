macro_rules! eval_float_binop {
    ($left:expr, $right:expr, $f1:ident, $f2:ident, $out:expr) => {
        match ($left, $right) {
            (JsNumber($f1), JsNumber($f2)) => JsNumber($out),
            _ => JsNumber(std::f64::NAN)
        }
    }
}

macro_rules! eval_float_sign {
    ($name:expr, $e:expr, $f:ident, $op:expr, $state:expr) => {
        match eval_exp(*$e, &mut $state) {
            JsNumber($f) => JsNumber($op),
            // TODO: coerce other types to Number first
            _ => JsNumber(std::f64::NAN),
        }
    }
}

macro_rules! eval_float_post_op {
    ($e:expr, $f:ident, $new:expr, $state:expr) => {
        match *$e {
            Var(var) => match $state.get(&var) {
                Some(&JsNumber($f)) => {
                    $state.insert(var, JsNumber($new));
                    eval_exp(Float($f), &mut $state)
                }
                _ => panic!("undefined variable `{}`", var)
            },
            _ => panic!("Invalid left-hand side expression in postfix operation")
        }
    }
}

macro_rules! eval_float_pre_op {
    ($e:expr, $f:ident, $new:expr, $state:expr) => {
        match *$e {
            Var(var) => match $state.get(&var) {
                Some(&JsNumber($f)) => {
                    $state.insert(var, JsNumber($new));
                    eval_exp(Float($new), &mut $state)
                }
                _ => panic!("undefined variable `{}`", var)
            },
            _ => panic!("Invalid left-hand side expression in prefix operation")
        }
    }
}
