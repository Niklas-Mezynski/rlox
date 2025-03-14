use crate::{token::Token, token_type::TokenType};

pub fn error(line: usize, message: &str) {
    report(line, "", message);
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
    unsafe {
        super::HAD_ERROR = true;
    }
}
