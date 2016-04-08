#[cfg(test)]
mod benchmarks {
    use ::eval::eval_string;
    use std::cell::RefCell;
    use std::rc::Rc;
    use french_press::init_gc;
    use test::Bencher;

    #[bench]
    fn simple_example(b: &mut Bencher) {
        let code = "\
        var x = 3;
        var y = 4;
        var z = x + y;
        ";

        let sm = Rc::new(RefCell::new(init_gc()));
        b.iter(|| eval_string(code, sm.clone()));
    }

    #[bench]
    fn complex_example(b: &mut Bencher) {
        let code = "\
        var x = 3;
        var y = 4;
        var z = 1000;
        while (z > 0) {
            z--;
            if (x < y) {
                x = x + 3;
            } else {
                x = x - y;
            }
        }
        ";

        let sm = Rc::new(RefCell::new(init_gc()));
        b.iter(|| eval_string(code, sm.clone()));
    }
}
