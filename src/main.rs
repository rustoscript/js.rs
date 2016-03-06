#![feature(test)]

extern crate js_types;
extern crate jsrs_common;
extern crate jsrs_parser;
extern crate french_press;
extern crate test;
extern crate walkdir;
extern crate unescape;

extern crate rustyline;

mod bench;
mod coerce;
mod eval;

use std::env;
use std::io::prelude::*;
use std::process::exit;
use std::io::{self, BufReader};
use std::path::Path;
use std::fs::{File, metadata};

use rustyline::error::ReadlineError;
use rustyline::Editor;

use walkdir::WalkDir;

use js_types::js_var::JsPtrEnum;

use french_press::{init_gc, ScopeManager};
use jsrs_common::ast::Exp;

use eval::eval_string;


fn eval_file(filename: String, debug: bool, should_repl: bool,
             mut scope_manager: &mut ScopeManager) {
    println!("Reading from \"{}\"", filename);
    let path = Path::new(&filename);
    let file = File::open(&path)
        .expect(&format!("Cannot open \"{}\": no such file or directory", filename));
    let file_buffer = BufReader::new(file);

    for (i, line) in file_buffer.lines().enumerate() {
        println!("{}: {:?}", i, line);
        let mut input = String::from(line
                                     .expect(&format!("Cannot read from {}", filename))
                                     .trim());

        if debug {
            println!(">> {}", input);
        }

        // insert semicolon if necessary
        if !input.ends_with(";") && !input.ends_with("}") {
            input.push_str(";");
        }

        // ignore comments
        if input.starts_with("//"){
            continue;
        }

        let ret = eval_string(&input, &mut scope_manager);
        if debug {
            println!("=> {:?}", ret);
        }
    }
    if should_repl {
        repl(scope_manager);
    }
}

fn test_dir() {
    let dir_name = "tests/numeric";

    for entry in WalkDir::new(dir_name) {
        let entry = entry.unwrap();
        if !entry.path().is_dir() {
            let entry_path = entry.path().display().to_string();
            let mut scope_manager = init_gc();
            eval_file(entry_path, false, false, &mut scope_manager);
        }
    }
}

fn repl(mut scope_manager: &mut ScopeManager) -> i32 {
    let mut rl = Editor::new();
    let mut stderr = io::stderr();

    scope_manager.push_scope(&Exp::Undefined);

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

                let (var, ptr) = eval_string(&input, &mut scope_manager);

                match ptr {
                    Some(JsPtrEnum::JsSym(s)) => println!("=> Symbol({:?})", s),
                    Some(JsPtrEnum::JsStr(s)) => println!("=> {:?}", s.text),
                    _ => println!("=> {:?}", var.t),
                }
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
    test_dir();
    let mut scope_manager = init_gc();

    let args = env::args();
    if args.len() > 1 {
        for file in args.skip(1) {
            eval_file(file, true, true, &mut scope_manager);
        }
    } else {
        let ret = repl(&mut scope_manager);
        exit(ret)
    }
}
