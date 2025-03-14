use crate::{
    error,
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
    pub token: Token,
    pub message: String,
}

impl RuntimeError {
    pub fn new(token: Token, message: String) -> RuntimeError {
        RuntimeError { token, message }
    }
}

pub trait Interpreter {
    fn interpret(self);
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

    fn stringify(self) -> String {
        match self {
            LoxValue::Nil => String::from("nul"),
            LoxValue::Boolean(value) => value.to_string(),
            LoxValue::Number(value) => value.to_string(),
            LoxValue::String(value) => value,
        }
    }
}

impl Interpreter for Expr {
    fn interpret(self) {
        let result = self.evaluate();

        match result {
            Ok(val) => {
                println!("{}", val.stringify())
            }
            Err(err) => {
                error::runtime_error(err);
            }
        }
    }
}

trait Evaluatable {
    fn evaluate(self) -> Result<LoxValue, RuntimeError>;
}

impl Evaluatable for Expr {
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
                            if right_num == 0_f64 {
                                return Err(RuntimeError::new(
                                    operator,
                                    "Cannot divide by 0.".to_string(),
                                ));
                            }
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
                    TokenType::Plus => {
                        match (left, right) {
                            (LoxValue::Number(left_num), LoxValue::Number(right_num)) => {
                                Ok(LoxValue::Number(left_num + right_num))
                            }

                            // If either one of the values is a str, we cast the other one to a string
                            (LoxValue::String(left_str), right_val) => Ok(LoxValue::String(
                                format!("{}{}", left_str, right_val.stringify()),
                            )),
                            (left_val, LoxValue::String(right_str)) => Ok(LoxValue::String(
                                format!("{}{}", left_val.stringify(), right_str),
                            )),

                            _ => Err(RuntimeError::new(
                                operator,
                                "Operands must be two numbers or two strings.".to_string(),
                            )),
                        }
                    }

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
            Expr::Conditional {
                condition,
                then,
                r#else,
            } => {
                let condition = condition.evaluate()?;

                if condition.is_truthy() {
                    then.evaluate()
                } else {
                    r#else.evaluate()
                }
            }
        }
    }
}
