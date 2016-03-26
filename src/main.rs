#![feature(plugin)]
#![plugin(docopt_macros)]

extern crate jsrs_common;
extern crate jsrs_parser;
extern crate french_press;

extern crate walkdir;
extern crate unescape;
extern crate rustyline;
extern crate rustc_serialize;
extern crate docopt;

mod bench;
mod eval;
mod native;
mod preprocess;

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

use jsrs_common::ast::Exp;
use jsrs_common::types::js_var::JsPtrEnum;
use french_press::{init_gc, ScopeManager};

use eval::eval_string;
use native::add_pervasives;
use preprocess::clean_string;

docopt!(Args derive Debug, "
js.rs - a javascript interpreter

Usage:
jsrs
jsrs <file>
jsrs --test
");

fn eval_file(filename: String, debug: bool, should_repl: bool,
             scope_manager: Rc<RefCell<ScopeManager>>) {
    println!("Reading from \"{}\"", filename);
    let path = Path::new(&filename);
    let file = File::open(&path)
        .expect(&format!("Cannot open \"{}\": no such file or directory", filename));
    let file_buffer = BufReader::new(file);

    let mut line_builder = String::new();
    let mut braces = Vec::new();

    let mut file_iter = file_buffer.lines();
    loop {
        if let Some(line) = file_iter.next() {
            let input = String::from(line.expect(&format!("Cannot read from {}", filename))
                                         .trim());
            if let Some(input) = clean_string(input) {
                //println!("{}", input);

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
                            panic!("parse error: unexpected token {}", c);
                        }
                    } else if c == '}' {
                        if braces.pop() != Some('{') {
                            panic!("parse error: unexpected token {}", c);
                        }
                    } else if c == '/' && last == '*' {
                        if braces.pop() != Some('/') {
                            panic!("parse error: unexpected token {}", c);
                        }
                    }
                    last = c;
                }

                line_builder.push_str(&input);

                if braces.len() == 0 {
                    clean_string(line_builder.clone()).map(|js_string| {
                        println!("{:?}\n", line_builder);
                        line_builder = String::new();
                        if debug {
                            println!(">> {}", js_string);
                        }

                        let ret = eval_string(&js_string, scope_manager.clone());
                        if debug {
                            println!("=> {:?}", ret);
                        }
                    });
                }
            }
        } else {
            break;
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
            let scope_manager = Rc::new(RefCell::new(init_gc()));
            eval_file(entry_path, false, false, scope_manager.clone());
        }
    }
}

fn repl(scope_manager: Rc<RefCell<ScopeManager>>) -> i32 {
    let mut rl = Editor::new();
    let mut stderr = io::stderr();

    scope_manager.borrow_mut().push_scope(&Exp::Undefined);

    if metadata(".history").is_ok() && rl.load_history(".history").is_err() {
        writeln!(stderr, "Error: unable to load history on startup").unwrap();
    }

    loop {
        // prompt

        let readline = rl.readline(">> ");

        match readline {
            Ok(line) => {
                rl.add_history_entry(&line);
                let input = String::from(line.trim());
                clean_string(input).map(|input| {
                    rl.add_history_entry(&input);

                    let (var, ptr) = eval_string(&input, scope_manager.clone());

                    match ptr {
                        Some(JsPtrEnum::JsSym(s)) => println!("=> Symbol({:?})", s),
                        Some(JsPtrEnum::JsStr(s)) => println!("=> {:?}", s.text),
                        _ => println!("=> {:?}", var.t),
                    }
                });
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
        test_dir()
    } else {
        let scope_manager = Rc::new(RefCell::new(init_gc()));
        add_pervasives(scope_manager.clone());

        if args.arg_file == "" {
            let ret = repl(scope_manager.clone());
            exit(ret)
        } else {
            eval_file(args.arg_file, true, true, scope_manager.clone());
        }
    }
}
