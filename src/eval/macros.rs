/// Compares two values as JsNum and returns a JsBool.
macro_rules! eval_cmp {
    ($left:expr, $right:expr, $f1:ident, $f2:ident, $out:expr) => {
        match ($left.as_number(), $right.as_number()) {
            (JsNum($f1), JsNum($f2)) => JsBool($out),
            _ => JsBool(false)
        }
    }
}

macro_rules! eval_float_sign {
    ($name:expr, $e:expr, $f:ident, $op:expr, $state:expr) => {
        match eval_exp(&*$e, &mut $state) {
            JsNum($f) => JsNum($op),
            // TODO: coerce other types to Number first
            _ => JsNum(std::f64::NAN),
        }
    }
}

/// eval_float_post_op!(exp, f, f - 1.0, state),
macro_rules! eval_float_post_op {
    ($e:expr, $f:ident, $new:expr, $state:expr) => {
        match **$e {
            Var(ref var) => match $state.get(var) {
                Some(&JsNum($f)) => {
                    $state.insert(var.clone(), JsNum($new));
                    eval_exp(&Float($f), &mut $state)
                }
                _ => panic!("undefined variable `{}`", var)
            },
            _ => panic!("Invalid left-hand side expression in postfix operation")
        }
    }
}

macro_rules! eval_float_pre_op {
    ($e:expr, $f:ident, $new:expr, $state:expr) => {
        match **$e {
            Var(ref var) => match $state.get(var) {
                Some(&JsNum($f)) => {
                    $state.insert(var.clone(), JsNum($new));
                    eval_exp(&Float($new), &mut $state)
                }
                _ => panic!("undefined variable `{}`", var)
            },
            _ => panic!("Invalid left-hand side expression in prefix operation")
        }
    }
}
