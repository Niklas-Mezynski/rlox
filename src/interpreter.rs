use crate::{
    expr::Expr,
    token::{Literal, Token},
    token_type::TokenType,
};

#[derive(Debug, PartialEq)]
pub enum LoxValue {
    String(String),
    Number(f64),
    Nil,
    Boolean(bool),
}

pub struct RuntimeError {
    token: Token,
    message: String,
}

impl RuntimeError {
    pub fn new(token: Token, message: String) -> RuntimeError {
        RuntimeError { token, message }
    }
}

pub trait Interpreter {
    fn evaluate(self) -> Result<LoxValue, RuntimeError>;
}

impl Literal {
    fn into(self) -> LoxValue {
        match self {
            Literal::Boolean(value) => LoxValue::Boolean(value),
            Literal::Nil => LoxValue::Nil,
            Literal::Number(value) => LoxValue::Number(value),
            Literal::String(value) => LoxValue::String(value),
        }
    }
}

impl LoxValue {
    fn is_truthy(&self) -> bool {
        match self {
            LoxValue::Boolean(val) => *val,
            LoxValue::Nil => false,
            _ => true,
        }
    }
}

impl Interpreter for Expr {
    fn evaluate(self) -> Result<LoxValue, RuntimeError> {
        match self {
            Expr::Literal { value } => Ok(value.into()),
            Expr::Grouping { expression } => expression.evaluate(),
            Expr::Unary { operator, right } => {
                let right = right.evaluate()?;

                match operator.token_type {
                    TokenType::Minus => match right {
                        LoxValue::Number(num) => Ok(LoxValue::Number(-num)),
                        _ => Err(RuntimeError::new(
                            operator,
                            "Cannot negate non numeric value".to_string(),
                        )),
                    },
                    TokenType::Bang => Ok(LoxValue::Boolean(!right.is_truthy())),
                    _ => panic!("Invalid unary expression in AST"),
                }
            }
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let left = left.evaluate()?;
                let right = right.evaluate()?;

                match operator.token_type {
                    // Arithmetic operations
                    TokenType::Minus => match (left, right) {
                        (LoxValue::Number(left_num), LoxValue::Number(right_num)) => {
                            Ok(LoxValue::Number(left_num - right_num))
                        }
                        _ => Err(RuntimeError::new(
                            operator,
                            "Operands must be numbers.".to_string(),
                        )),
                    },
                    TokenType::Slash => match (left, right) {
                        (LoxValue::Number(left_num), LoxValue::Number(right_num)) => {
                            Ok(LoxValue::Number(left_num / right_num))
                        }
                        _ => Err(RuntimeError::new(
                            operator,
                            "Operands must be numbers.".to_string(),
                        )),
                    },
                    TokenType::Star => match (left, right) {
                        (LoxValue::Number(left_num), LoxValue::Number(right_num)) => {
                            Ok(LoxValue::Number(left_num * right_num))
                        }
                        _ => Err(RuntimeError::new(
                            operator,
                            "Operands must be numbers.".to_string(),
                        )),
                    },
                    TokenType::Plus => match (left, right) {
                        (LoxValue::Number(left_num), LoxValue::Number(right_num)) => {
                            Ok(LoxValue::Number(left_num + right_num))
                        }
                        (LoxValue::String(left_str), LoxValue::String(right_str)) => {
                            Ok(LoxValue::String(format!("{}{}", left_str, right_str)))
                        }
                        _ => Err(RuntimeError::new(
                            operator,
                            "Operands must be two numbers or two strings.".to_string(),
                        )),
                    },

                    // Comparison operations
                    TokenType::Greater => match (left, right) {
                        (LoxValue::Number(left_num), LoxValue::Number(right_num)) => {
                            Ok(LoxValue::Boolean(left_num > right_num))
                        }
                        _ => Err(RuntimeError::new(
                            operator,
                            "Operands must be numbers.".to_string(),
                        )),
                    },
                    TokenType::GreaterEqual => match (left, right) {
                        (LoxValue::Number(left_num), LoxValue::Number(right_num)) => {
                            Ok(LoxValue::Boolean(left_num >= right_num))
                        }
                        _ => Err(RuntimeError::new(
                            operator,
                            "Operands must be numbers.".to_string(),
                        )),
                    },
                    TokenType::Less => match (left, right) {
                        (LoxValue::Number(left_num), LoxValue::Number(right_num)) => {
                            Ok(LoxValue::Boolean(left_num < right_num))
                        }
                        _ => Err(RuntimeError::new(
                            operator,
                            "Operands must be numbers.".to_string(),
                        )),
                    },
                    TokenType::LessEqual => match (left, right) {
                        (LoxValue::Number(left_num), LoxValue::Number(right_num)) => {
                            Ok(LoxValue::Boolean(left_num <= right_num))
                        }
                        _ => Err(RuntimeError::new(
                            operator,
                            "Operands must be numbers.".to_string(),
                        )),
                    },

                    // Equality operations
                    TokenType::BangEqual => Ok(LoxValue::Boolean(left != right)),
                    TokenType::EqualEqual => Ok(LoxValue::Boolean(left == right)),
                    _ => Err(RuntimeError::new(
                        operator,
                        "Invalid binary operator.".to_string(),
                    )),
                }
            }
            _ => todo!(),
        }
    }
}
