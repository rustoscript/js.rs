extern crate jsrs_parser;

mod eval;

use std::io;
use std::collections::HashMap;
use eval::eval_string;

fn main() {
    let mut state = HashMap::new();

    loop {
        // read input
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        if input.len() == 0 {
            break;
        }

        println!("{:?}", eval_string(&input, &mut state));
        println!("{:?}", state);
    }
}
