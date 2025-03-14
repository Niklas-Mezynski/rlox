use crate::{error, token::Token, token_type::TokenType};

#[derive(Debug)]
pub struct Scanner {
    source: String,
    source_chars: Vec<char>,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Scanner {
            source_chars: source.chars().collect(),
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(mut self) -> Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }

        self.tokens
            .push(Token::new(TokenType::Eof, String::from(""), self.line));

        self.tokens
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source_chars.len()
    }

    fn scan_token(&mut self) {
        let c = self.advance();

        match c {
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            '-' => self.add_token(TokenType::Minus),
            '+' => self.add_token(TokenType::Plus),
            ';' => self.add_token(TokenType::Semicolon),
            '*' => self.add_token(TokenType::Star),
            '?' => self.add_token(TokenType::QuestionMark),
            ':' => self.add_token(TokenType::Colon),

            '!' => {
                if self.matches('=') {
                    self.add_token(TokenType::BangEqual)
                } else {
                    self.add_token(TokenType::Bang)
                }
            }
            '=' => {
                if self.matches('=') {
                    self.add_token(TokenType::EqualEqual)
                } else {
                    self.add_token(TokenType::Equal)
                }
            }
            '<' => {
                if self.matches('=') {
                    self.add_token(TokenType::LessEqual)
                } else {
                    self.add_token(TokenType::Less)
                }
            }
            '>' => {
                if self.matches('=') {
                    self.add_token(TokenType::GreaterEqual)
                } else {
                    self.add_token(TokenType::Greater)
                }
            }
            '/' => {
                // Comments
                if self.matches('/') {
                    // Consume chars until the end of the line
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::Slash);
                }
            }

            ' ' | '\r' | '\t' => {} // Ignore whitespace
            '\n' => self.line += 1, // Increment line number

            '"' => self.string(),

            '0'..='9' => self.number(),

            c if c.is_alpha_lox() => self.identifier(),

            c => {
                error::error(
                    self.line,
                    format!("Unexpected character: '{}'.", c).as_str(),
                );
            }
        }
    }

    fn identifier(&mut self) {
        while self.peek().is_alphanumeric_lox() {
            self.advance();
        }

        let text = self.source[self.start..self.current].to_string();

        match self.keyword(&text) {
            Some(token_type) => self.add_token(token_type),
            None => self.add_token(TokenType::Identifier),
        };
    }

    fn keyword(&mut self, text: &str) -> Option<TokenType> {
        match text {
            "and" => Some(TokenType::And),
            "class" => Some(TokenType::Class),
            "else" => Some(TokenType::Else),
            "false" => Some(TokenType::False),
            "for" => Some(TokenType::For),
            "fun" => Some(TokenType::Fun),
            "if" => Some(TokenType::If),
            "nil" => Some(TokenType::Nil),
            "or" => Some(TokenType::Or),
            "print" => Some(TokenType::Print),
            "return" => Some(TokenType::Return),
            "super" => Some(TokenType::Super),
            "this" => Some(TokenType::This),
            "true" => Some(TokenType::True),
            "var" => Some(TokenType::Var),
            "while" => Some(TokenType::While),
            _ => None,
        }
    }

    fn number(&mut self) {
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        // Look for a fractional part.
        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            // Consume the "."
            self.advance();

            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        self.add_token(TokenType::Number(
            self.source[self.start..self.current].parse().unwrap(),
        ));
    }

    fn string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }

            self.advance();
        }

        if self.is_at_end() {
            error::error(self.line, "Unterminated string.");
            return;
        }

        // The closing ".
        self.advance();

        // Trim the surrounding quotes.
        let value = self.source[self.start + 1..self.current - 1].to_string();
        self.add_token(TokenType::String(value));
    }

    fn matches(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }

        if self.source_chars[self.current] != expected {
            return false;
        }

        self.current += 1;
        true
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source_chars.len() {
            return '\0';
        }

        self.source_chars[self.current + 1]
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }

        self.source_chars[self.current]
    }

    fn advance(&mut self) -> char {
        let c = self.source_chars[self.current];
        self.current += 1;
        c
    }

    fn add_token(&mut self, token_type: TokenType) {
        let text = self.source[self.start..self.current].to_string();
        self.tokens.push(Token::new(token_type, text, self.line));
    }
}

trait Alpha {
    fn is_alpha_lox(&self) -> bool;
}

trait Alphanumeric {
    fn is_alphanumeric_lox(&self) -> bool;
}

impl Alpha for char {
    fn is_alpha_lox(&self) -> bool {
        self.is_ascii_alphabetic() || *self == '_'
    }
}

impl Alphanumeric for char {
    fn is_alphanumeric_lox(&self) -> bool {
        self.is_ascii_alphanumeric() || *self == '_'
    }
}

#[cfg(test)]
mod tests {
    use super::Scanner;
    use super::TokenType;

    #[test]
    fn test_scanner_single_character_tokens() {
        let source = String::from("(){},.-+;*");
        let tokens = Scanner::new(source).scan_tokens();

        let expected_types = vec![
            TokenType::LeftParen,
            TokenType::RightParen,
            TokenType::LeftBrace,
            TokenType::RightBrace,
            TokenType::Comma,
            TokenType::Dot,
            TokenType::Minus,
            TokenType::Plus,
            TokenType::Semicolon,
            TokenType::Star,
            TokenType::Eof,
        ];

        assert_eq!(tokens.len(), expected_types.len());
        for (token, expected_type) in tokens.iter().zip(expected_types.iter()) {
            assert_eq!(&token.token_type, expected_type);
        }
    }

    #[test]
    fn test_with_lox_file() {
        let source = std::fs::read_to_string("./test/lex.lox").expect("Failed to read file");

        let tokens = Scanner::new(source).scan_tokens();

        assert!(!tokens.is_empty());
    }
}
