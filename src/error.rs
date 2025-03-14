use crate::{interpreter::RuntimeError, token::Token, token_type::TokenType};

static mut HAD_ERROR: bool = false;
static mut HAD_RUNTIME_ERROR: bool = false;

pub fn error(line: usize, message: &str) {
    report(line, "", message);
}

pub fn runtime_error(error: RuntimeError) {
    eprintln!("[line {}]: {}", error.token.line, error.message);
    set_had_runtime_error(true);
}

pub fn had_error() -> bool {
    unsafe { HAD_ERROR }
}

pub fn set_had_error(had_error: bool) {
    unsafe { HAD_ERROR = had_error };
}

pub fn had_runtime_error() -> bool {
    unsafe { HAD_RUNTIME_ERROR }
}

fn set_had_runtime_error(had_runtime_error: bool) {
    unsafe { HAD_RUNTIME_ERROR = had_runtime_error };
}

pub fn error_token(token: &Token, message: &str) {
    match token.token_type {
        TokenType::Eof => report(token.line, " at end", message),
        _ => report(
            token.line,
            format!("at '{}'", token.lexeme).as_str(),
            message,
        ),
    }
}

fn report(line: usize, location: &str, message: &str) {
    eprintln!("[line {}] Error {}: {}", line, location, message);
    set_had_error(true);
}
