/// Evaluate numerical exp on two values as JsNumber and returns a JsNumber.
macro_rules! eval_num_binop {
    ($left:expr, $right:expr, $f1:ident, $f2:ident, $out:expr) => {
        match ($left.as_number(), $right.as_number()) {
            (JsNumber($f1), JsNumber($f2)) => JsNumber($out),
            _ => JsNumber(std::f64::NAN)
        }
    }
}

/// Compares two values as JsNumber and returns a JsBool.
macro_rules! eval_cmp {
    ($left:expr, $right:expr, $f1:ident, $f2:ident, $out:expr) => {
        match ($left.as_number(), $right.as_number()) {
            (JsNumber($f1), JsNumber($f2)) => JsBool($out),
            _ => JsBool(false)
        }
    }
}

/// Compares two values as JsBool and returns a JsBool.
macro_rules! eval_logic {
    ($left:expr, $right:expr, $f1:ident, $f2:ident, $out:expr) => {
        match ($left.as_bool(), $right.as_bool()) {
            (JsBool($f1), JsBool($f2)) => JsBool($out),
            _ => JsBool(false)
        }
    }
}

macro_rules! eval_float_sign {
    ($name:expr, $e:expr, $f:ident, $op:expr, $state:expr) => {
        match eval_exp(&*$e, &mut $state) {
            JsNumber($f) => JsNumber($op),
            // TODO: coerce other types to Number first
            _ => JsNumber(std::f64::NAN),
        }
    }
}

/// eval_float_post_op!(exp, f, f - 1.0, state),
macro_rules! eval_float_post_op {
    ($e:expr, $f:ident, $new:expr, $state:expr) => {
        match **$e {
            Var(ref var) => match $state.get(var) {
                Some(&JsNumber($f)) => {
                    $state.insert(var.clone(), JsNumber($new));
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
                Some(&JsNumber($f)) => {
                    $state.insert(var.clone(), JsNumber($new));
                    eval_exp(&Float($new), &mut $state)
                }
                _ => panic!("undefined variable `{}`", var)
            },
            _ => panic!("Invalid left-hand side expression in prefix operation")
        }
    }
}
