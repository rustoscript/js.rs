extern crate jsrs_parser;

mod value;
mod eval;

use std::env;
use std::io;
//use std::io::{self, Write};
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;
use std::fs::File;

use std::collections::HashMap;

use eval::eval_string;


fn eval_file(filename: String, debug: bool) {
    let mut state = HashMap::new();

    println!("Reading from \"{}\"", filename);
    let path = Path::new(&filename);
    let file = File::open(&path)
        .expect(&format!("Cannot open \"{}\": no such file or directory", filename));
    let file_buffer = BufReader::new(file);

    for line in file_buffer.lines() {
        let mut input = String::from(line.unwrap().trim());

        if debug {
            println!(">> {:?}", input);
        }

        // insert semicolon if necessary
        if !input.ends_with(";") && !input.ends_with("}") {
            input.push_str(";");
        }

        if debug {
            println!("=> {:?}", eval_string(&input, &mut state));
            println!("** {:?}", state);
        }
    }
}

fn repl() {
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

fn main() {
    let args = env::args();
    if args.len() > 1 {
        for file in args.skip(1) {
            eval_file(file, true);
        }
    } else {
        repl();
    }
}
