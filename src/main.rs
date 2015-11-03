extern crate jsrs_parser;

mod eval;

use std::io::{self, Write};
use std::collections::HashMap;
use eval::eval_string;

fn main() {
    let mut state = HashMap::new();
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        // prompt
        print!(">> ");
        stdout.flush().unwrap();

        // read input
        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();

        if input == "\n" {
            continue;
        }

        if !input.ends_with(";") && !input.ends_with("}") {
            input.push_str(";");
        }

        println!("=> {:?}", eval_string(&input, &mut state));
        println!("** {:?}", state);
    }
}
