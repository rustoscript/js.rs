extern crate jsrs_parser;

mod eval;

use std::io;
use eval::eval_string;

fn main() {
    loop {
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        if input.len() == 0 {
            break;
        }

        println!("{:?}", eval_string(&input));
    }
}
