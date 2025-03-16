use std::{cell::RefCell, rc::Rc};

use crate::{
    environment::Environment,
    error,
    expr::Expr,
    stmt::Stmt,
    token::{Literal, Token},
    token_type::TokenType,
};

#[derive(Debug, PartialEq, Clone)]
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

pub struct Interpreter {
    environment: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter {
            environment: Rc::new(RefCell::new(Environment::new())),
        }
    }

    pub fn interpret(&mut self, statements: Vec<Stmt>) {
        for statement in statements {
            if let Err(err) = statement.evaluate(self.environment.clone()) {
                error::runtime_error(err);
                return;
            }
        }
    }
}

impl Stmt {
    fn evaluate(self, environment: Rc<RefCell<Environment>>) -> Result<(), RuntimeError> {
        match self {
            Stmt::Expression { expr } => {
                expr.evaluate(environment)?;
                Ok(())
            }
            Stmt::Print { expr } => {
                let value = expr.evaluate(environment)?;
                println!("{}", value.stringify());
                Ok(())
            }
            Stmt::Var { name, initializer } => {
                let mut value = (LoxValue::Nil);

                if let Some(expr) = initializer {
                    value = expr.evaluate(environment.clone())?;
                }

                let owned_value = value;
                environment.borrow_mut().define(name.lexeme, owned_value);
                Ok(())
            }
        }
    }
}

impl Expr {
    fn evaluate(self, environment: Rc<RefCell<Environment>>) -> Result<LoxValue, RuntimeError> {
        match self {
            Expr::Literal { value } => Ok(value.into()),
            Expr::Grouping { expression } => expression.evaluate(environment),
            Expr::Unary { operator, right } => {
                let right = right.evaluate(environment)?;

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
                let left = left.evaluate(environment.clone())?;
                let left_value = left;

                let right = right.evaluate(environment)?;
                let right_value = right;

                match operator.token_type {
                    // Arithmetic operations
                    TokenType::Minus => match (left_value, right_value) {
                        (LoxValue::Number(left_num), LoxValue::Number(right_num)) => {
                            Ok(LoxValue::Number(left_num - right_num))
                        }
                        _ => Err(RuntimeError::new(
                            operator,
                            "Operands must be numbers.".to_string(),
                        )),
                    },
                    TokenType::Slash => match (left_value, right_value) {
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
                    TokenType::Star => match (left_value, right_value) {
                        (LoxValue::Number(left_num), LoxValue::Number(right_num)) => {
                            Ok(LoxValue::Number(left_num * right_num))
                        }
                        _ => Err(RuntimeError::new(
                            operator,
                            "Operands must be numbers.".to_string(),
                        )),
                    },
                    TokenType::Plus => {
                        match (left_value, right_value) {
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
                    TokenType::Greater => match (left_value, right_value) {
                        (LoxValue::Number(left_num), LoxValue::Number(right_num)) => {
                            Ok(LoxValue::Boolean(left_num > right_num))
                        }
                        _ => Err(RuntimeError::new(
                            operator,
                            "Operands must be numbers.".to_string(),
                        )),
                    },
                    TokenType::GreaterEqual => match (left_value, right_value) {
                        (LoxValue::Number(left_num), LoxValue::Number(right_num)) => {
                            Ok(LoxValue::Boolean(left_num >= right_num))
                        }
                        _ => Err(RuntimeError::new(
                            operator,
                            "Operands must be numbers.".to_string(),
                        )),
                    },
                    TokenType::Less => match (left_value, right_value) {
                        (LoxValue::Number(left_num), LoxValue::Number(right_num)) => {
                            Ok(LoxValue::Boolean(left_num < right_num))
                        }
                        _ => Err(RuntimeError::new(
                            operator,
                            "Operands must be numbers.".to_string(),
                        )),
                    },
                    TokenType::LessEqual => match (left_value, right_value) {
                        (LoxValue::Number(left_num), LoxValue::Number(right_num)) => {
                            Ok(LoxValue::Boolean(left_num <= right_num))
                        }
                        _ => Err(RuntimeError::new(
                            operator,
                            "Operands must be numbers.".to_string(),
                        )),
                    },

                    // Equality operations
                    TokenType::BangEqual => Ok(LoxValue::Boolean(left_value != right_value)),
                    TokenType::EqualEqual => Ok(LoxValue::Boolean(left_value == right_value)),
                    _ => Err(RuntimeError::new(
                        operator,
                        "Invalid binary operator.".to_string(),
                    )),
                }
            }
            Self::Variable { name } => environment.borrow().get(&name).cloned(),
            Expr::Assign { name, value } => {
                let value = value.evaluate(environment.clone())?;
                environment.borrow_mut().assign(&name, value.clone())?;
                Ok(value)
            }
            Expr::Conditional {
                condition,
                then,
                r#else,
            } => {
                let condition = condition.evaluate(environment.clone())?;

                if condition.is_truthy() {
                    then.evaluate(environment)
                } else {
                    r#else.evaluate(environment)
                }
            }
        }
    }
}
