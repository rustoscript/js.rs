extern crate jsrs_parser;

#[macro_use]
mod macros;

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

        // exit if no input (eg. ^D)
        if input.len() == 0 {
            println!("");
            break;
        }

        input = String::from(input.trim());

        // ignore if only whitespace
        if input.len() == 0 {
            continue;
        }

        // insert semicolon if necessary
        if !input.ends_with(";") && !input.ends_with("}") {
            input.push_str(";");
        }

        // eval
        println!("=> {:?}", eval_string(&input, &mut state));
        println!("** {:?}", state);
    }
}
