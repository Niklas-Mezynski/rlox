use std::{env, io::Write};

use interpreter::Interpreter;
use parser::Parser;
use resolver::{Resolvable, Resolver};
use scanner::Scanner;

mod ast_printer;
mod environment;
mod error;
mod expr;
mod interpreter;
mod lox_callable;
mod lox_instance;
mod parser;
mod resolver;
mod scanner;
mod stmt;
mod token;
mod token_type;
fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 2 {
        println!("Usage: rlox [script]");
        std::process::exit(1);
    }

    let mut interpreter = Interpreter::new();

    if args.len() == 2 {
        run_file(&args[1], &mut interpreter);
    } else {
        run_prompt(&mut interpreter);
    }
}

fn run_file(path: &str, interpreter: &mut Interpreter) {
    let source = std::fs::read_to_string(path).expect("Failed to read file");

    run(source, interpreter);

    if error::had_error() {
        std::process::exit(65);
    }
    if error::had_runtime_error() {
        std::process::exit(70);
    }
}

fn run_prompt(interpreter: &mut Interpreter) {
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

        run(input, interpreter);
        std::io::stdout().flush().expect("Cannot flush stdout");

        error::set_had_error(false);
    }
}

fn run(source: String, interpreter: &mut Interpreter) {
    let tokens = Scanner::new(source).scan_tokens();
    let expr = Parser::new(tokens).parse();

    // Check if we had error during parsing
    if error::had_error() {
        return;
    }

    let mut statements = expr.expect("Should have expression as there was no error reported");

    let mut resolver = Resolver::new();
    statements.resolve(&mut resolver);

    // Check again after resolution
    if error::had_error() {
        return;
    }

    interpreter.interpret(statements);
}
