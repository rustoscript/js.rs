macro_rules! eval_float_binop {
    ($left:expr, $right:expr, $f1:ident, $f2:ident, $out:expr) => {
        match ($left, $right) {
            (Float($f1), Float($f2)) => Float($out),
            _ => Float(std::f64::NAN)
        }
    }
}

macro_rules! eval_float_sign {
    ($name:expr, $e:expr, $f:ident, $op:expr, $state:expr) => {
        match eval_exp(*$e, &mut $state) {
            BinExp(..) => panic!("error in {}: `eval` should never return BinExp", $name),
            Float($f) => Float($op),
            Neg(..) => panic!("error in {}: `eval` should never return Neg", $name),
            Pos(..) => panic!("error in {}: `eval` should never return Pos", $name),
            PostDec(..) => panic!("error in {}: `eval` should never return PostDec", $name),
            PostInc(..) => panic!("error in {}: `eval` should never return PostInc", $name),
            PreDec(..) => panic!("error in {}: `eval` should never return PreDec", $name),
            PreInc(..) => panic!("error in {}: `eval` should never return PreInc", $name),
            Undefined => Float(std::f64::NAN),
            Var(..) => panic!("error in {}: `eval` should never return Var", $name),
        }
    }
}

macro_rules! eval_float_post_op {
    ($err:expr, $e:expr, $f:ident, $new:expr, $state:expr) => {
        match *$e {
            Var(var) => match $state.get(&var) {
                Some(&Float($f)) => {
                    $state.insert(var, Float($new));
                    eval_exp(Float($f), &mut $state)
                }
                _ => panic!("undefined variable `{}`", var)
            },
            _ => panic!("{}", $err)
        }
    }
}

macro_rules! eval_float_pre_op {
    ($err:expr, $e:expr, $f:ident, $new:expr, $state:expr) => {
        match *$e {
            Var(var) => match $state.get(&var) {
                Some(&Float($f)) => {
                    $state.insert(var, Float($new));
                    eval_exp(Float($new), &mut $state)
                }
                _ => panic!("undefined variable `{}`", var)
            },
            _ => panic!("{}", $err)
        }
    }
}
