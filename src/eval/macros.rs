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
