

#[cfg(test)]
mod benchmarks {
    use ::eval::eval_string;
    use french_press::init_gc;
    use test::Bencher;

    #[bench]
    fn bench_example(b: &mut Bencher) {
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
        let mut sm = init_gc();

        b.iter(|| eval_string(code, &mut sm));
    }
}
