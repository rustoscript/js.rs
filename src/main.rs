extern crate jsrs_parser;
extern crate jsrs_common;
extern crate rustyline;

mod value;
mod eval;
mod js_value;

use std::env;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;
use std::fs::File;

use std::collections::HashMap;

use rustyline::error::ReadlineError;
use rustyline::Editor;

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
    let mut rl = Editor::new();

    loop {
        // prompt
        let readline = rl.readline(">> ");

        match readline {
            Ok(line) => {
                rl.add_history_entry(&line);
                let mut input = String::from(line.trim());

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
                println!("Line: {}", line);
            },
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break
            },
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        }
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
