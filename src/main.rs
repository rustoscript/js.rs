#![feature(plugin)]
#![plugin(docopt_macros)]

#![feature(test)]

extern crate jsrs_common;
extern crate jsrs_parser;
extern crate french_press;

extern crate walkdir;
extern crate unescape;
extern crate rustyline;
extern crate rustc_serialize;
extern crate docopt;

extern crate test;

mod js_error;

mod eval;
mod native;
mod number;
mod bench;
mod preprocess;
mod var;

use std::cell::RefCell;
use std::fs::{File, metadata};
use std::io::{self, BufReader};
use std::io::prelude::*;
use std::path::Path;
use std::process::exit;
use std::rc::Rc;

use rustyline::error::ReadlineError;
use rustyline::Editor;
use walkdir::WalkDir;

use jsrs_common::types::js_var::JsPtrEnum;
use french_press::{init_gc, ScopeManager};

use eval::eval_string;
use native::add_pervasives;
use preprocess::{clean_string, add_semicolon};
use js_error::JsError;


docopt!(Args derive Debug, "
js.rs - a javascript interpreter

Usage:
jsrs
jsrs <file>
jsrs --test
");

fn eval_file(filename: String, debug: bool, should_repl: bool,
             scope_manager: Rc<RefCell<ScopeManager>>) -> js_error::Result<()> {
    if debug {
        println!("Reading from \"{}\"", filename);
    }

    let path = Path::new(&filename);
    let file = File::open(&path)
        .expect(&format!("Cannot open \"{}\": no such file or directory", filename));
    let file_buffer = BufReader::new(file);

    let mut line_builder = String::new();
    let mut braces = Vec::new();

    let mut file_iter = file_buffer.lines();
    let mut negative_test = false;
    loop {
        if let Some(line) = file_iter.next() {
            let input = String::from(line.expect(&format!("Cannot read from {}", filename)));
            let input = clean_string(input);
            if input.contains("@negative") {
                negative_test = true;
            }

            let mut last = '\0';
            // Match braces to see if we should wait for more input
            for c in input.chars() {
                if c == '(' {
                    braces.push('(');
                } else if c == '{' {
                    braces.push('{');
                } else if c == '*' && last == '/' {
                    braces.push('/');
                } else if c == ')' {
                    if braces.pop() != Some('(') {
                        return Err(JsError::ParseError(format!("Unexpected token {}", c)));
                    }
                } else if c == '}' {
                    if braces.pop() != Some('{') {
                        return Err(JsError::ParseError(format!("Unexpected token {}", c)));
                    }
                } else if c == '/' && last == '*' {
                    if braces.pop() != Some('/') {
                        return Err(JsError::ParseError(format!("Unexpected token {}", c)));
                    }
                }
                last = c;
            }

            line_builder.push_str(&input);

            if braces.len() == 0 {
                let js_string = clean_string(line_builder.clone());
                line_builder = String::new();
                if js_string == "" {
                    continue;
                }

                if debug {
                    println!("\n{}", js_string);
                }

                let ret;
                let eval_result = eval_string(&js_string, scope_manager.clone());
                if negative_test {
                    match eval_result {
                        Ok((var, ptr)) => ret = (var, ptr),
                        Err(e) => {
                            if !e.is_meta_error() {
                                continue;
                            } else {
                                return Err(e);
                            }
                        }
                    }
                } else {
                    ret = try!(eval_result);
                }

                if debug {
                    println!("=> {:?}", ret);
                }
            }
        } else {
            break;
        }
    }
    if should_repl {
        repl(scope_manager);
    }
    Ok(())
}

fn test_dir(dir_name: String) {
    for entry in WalkDir::new(dir_name) {
        let entry = entry.unwrap();
        if !entry.path().is_dir() {
            let entry_path = entry.path().display().to_string();
            let scope_manager = Rc::new(RefCell::new(init_gc()));
            add_pervasives(scope_manager.clone());
            match eval_file(entry_path.clone(), false, false, scope_manager.clone()) {
                Ok(_) => {
                    println!("{}: {}", entry_path, "OK");
                }
                Err(e) => {
                    println!("{}: {}", entry_path, e);
                }
            }
        }
    }
}

fn repl(scope_manager: Rc<RefCell<ScopeManager>>) -> i32 {
    let mut rl = Editor::new();
    let mut stderr = io::stderr();

    if metadata(".history").is_ok() && rl.load_history(".history").is_err() {
        writeln!(stderr, "Error: unable to load history on startup").unwrap();
    }

    loop {
        // prompt
        let readline = rl.readline(">> ");

        match readline {
            Ok(line) => {
                rl.add_history_entry(&line);
                let input = add_semicolon(clean_string(String::from(line)));
                if input == "" {
                    continue;
                }
                rl.add_history_entry(&input);

                match eval_string(&input, scope_manager.clone()) {
                    Ok((var, ptr)) => {
                        match ptr {
                            Some(JsPtrEnum::JsSym(s)) => println!("=> Symbol({:?})", s),
                            Some(JsPtrEnum::JsStr(s)) => println!("=> {:?}", s.text),
                            _ => println!("=> {:?}", var.t),
                        }
                    },
                    Err(e) => {
                        println!("{:?}", e);
                    }
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
    let args: Args = Args::docopt().decode().unwrap_or_else(|e| e.exit());
    if args.flag_test {
        let dir_name = "sputnik";

        test_dir(String::from(dir_name))
    } else {
        let scope_manager = Rc::new(RefCell::new(init_gc()));
        add_pervasives(scope_manager.clone());

        if args.arg_file == "" {
            let ret = repl(scope_manager.clone());
            exit(ret)
        } else {
            eval_file(args.arg_file, true, true, scope_manager.clone())
                .expect("Error evaluating file");
        }
    }
}
