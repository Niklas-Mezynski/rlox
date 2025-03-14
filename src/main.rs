use std::{env, io::Write};

use crate::token_type::TokenType;
use ast_printer::AstPrinter;
use parser::Parser;
use scanner::Scanner;

mod ast_printer;
mod error;
mod expr;
mod interpreter;
mod parser;
mod scanner;
mod token;
mod token_type;

pub static mut HAD_ERROR: bool = false;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 2 {
        println!("Usage: rlox [script]");
        std::process::exit(1);
    }

    if args.len() == 2 {
        run_file(&args[1]);
    } else {
        run_prompt();
    }
}

fn run_file(path: &str) {
    let source = std::fs::read_to_string(path).expect("Failed to read file");

    run(source);

    if unsafe { HAD_ERROR } {
        std::process::exit(65);
    }
}

fn run_prompt() {
    loop {
        print!("> ");
        std::io::stdout().flush().expect("Cannot flush stdout");

        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        if input.is_empty() {
            break;
        }

        run(input);
        std::io::stdout().flush().expect("Cannot flush stdout");

        unsafe {
            HAD_ERROR = false;
        }
    }
}

fn run(source: String) {
    let tokens = Scanner::new(source).scan_tokens();
    let expr = Parser::new(tokens).parse();

    if unsafe { HAD_ERROR } {
        return;
    }

    let expr = expr.unwrap();
    println!("{}", expr.print());
}
