extern crate jsrs_parser;
extern crate jsrs_common;
extern crate french_press;
extern crate rustyline;
extern crate uuid;

mod coerce;
mod state;
mod value;
mod eval;

use std::env;
use std::io::prelude::*;
use std::process::exit;
use std::io::{self, BufReader};
use std::path::Path;
use std::fs::{File, metadata};

use std::collections::HashMap;

use rustyline::error::ReadlineError;
use rustyline::Editor;

use state::StateManager;

use french_press::init_gc;
use std::collections::hash_set::HashSet;
use uuid::Uuid;

use eval::eval_string;


fn eval_file(filename: String, debug: bool) {
    let mut state = StateManager::new();

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

fn new_hash_set() -> HashSet<Uuid> {
    HashSet::new()
}

fn repl() -> i32 {
    let mut state = StateManager::new();
    let mut rl = Editor::new();
    let mut stderr = io::stderr();

    let scope_manager = init_gc(new_hash_set);

    if metadata(".history").is_ok() && rl.load_history(".history").is_err() {
        writeln!(stderr, "Error: unable to load history on startup").unwrap();
    }

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

                rl.add_history_entry(&input);

                // insert semicolon if necessary
                if !input.ends_with(";") && !input.ends_with("}") {
                    input.push_str(";");
                }

                // eval
                println!("=> {:?}", eval_string(&input, &mut state));
                println!("** {:?}", state);
            },
            Err(ReadlineError::Interrupted) => {
                if rl.save_history(".history").is_err() {
                    writeln!(stderr, "Error: unable to save history on exit").unwrap();
                    return 2;
                }
                return 1;
            },
            Err(ReadlineError::Eof) => {
                if rl.save_history(".history").is_err() {
                    writeln!(stderr, "Error: unable to save history on exit").unwrap();
                    return 2;
                }
                break
            },
            Err(err) => {
                println!("Error: {:?}", err);

                if rl.save_history(".history").is_err() {
                    writeln!(stderr, "Error: unable to save history on exit").unwrap();
                    return 2;
                }
                return 3;
            }
        }
    }
    0
}

fn main() {
    let args = env::args();
    if args.len() > 1 {
        for file in args.skip(1) {
            eval_file(file, true);
        }
    } else {
        let ret = repl();
        exit(ret)
    }
}
