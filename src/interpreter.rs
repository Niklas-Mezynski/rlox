use std::{cell::RefCell, fmt::Debug, rc::Rc};

use crate::{
    environment::Environment,
    error,
    expr::Expr,
    lox_callable::{FunctionStmt, LoxCallable},
    stmt::Stmt,
    token::{Literal, Token},
    token_type::TokenType,
};

#[derive(Debug)]
pub enum LoxValue {
    String(String),
    Number(f64),
    Nil,
    Boolean(bool),
    Callable(LoxCallable),
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
    fn into(&self) -> LoxValue {
        match self {
            Literal::Boolean(value) => LoxValue::Boolean(*value),
            Literal::Nil => LoxValue::Nil,
            Literal::Number(value) => LoxValue::Number(*value),
            Literal::String(value) => LoxValue::String(value.to_owned()),
        }
    }
}

pub trait Stringifyable {
    fn stringify(&self) -> String;
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

impl PartialEq for LoxValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::String(l0), Self::String(r0)) => l0 == r0,
            (Self::Number(l0), Self::Number(r0)) => l0 == r0,
            (Self::Boolean(l0), Self::Boolean(r0)) => l0 == r0,
            // TODO: How should this behave for functions etc.
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl Stringifyable for LoxValue {
    fn stringify(&self) -> String {
        match self {
            LoxValue::Nil => String::from("nul"),
            LoxValue::Boolean(value) => value.to_string(),
            LoxValue::Number(value) => value.to_string(),
            LoxValue::String(value) => value.clone(),
            LoxValue::Callable(value) => value.stringify(),
        }
    }
}

pub struct Interpreter {
    environment: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        let global_env = Rc::new(RefCell::new(Environment::new()));

        global_env.borrow_mut().define(
            "clock".to_string(),
            Rc::new(LoxValue::Callable(LoxCallable::ClockFunction)),
        );

        Interpreter {
            environment: global_env,
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

pub trait Evaluatable<T> {
    fn evaluate(&self, environment: Rc<RefCell<Environment>>) -> Result<T, RuntimeError>;
}

impl Evaluatable<()> for Stmt {
    fn evaluate(&self, environment: Rc<RefCell<Environment>>) -> Result<(), RuntimeError> {
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
                let mut value = Rc::new(LoxValue::Nil);

                if let Some(expr) = initializer {
                    value = expr.evaluate(environment.clone())?;
                }

                environment.borrow_mut().define(name.lexeme.clone(), value);
                Ok(())
            }
            Stmt::Block { statements } => {
                statements.evaluate(Rc::new(RefCell::new(Environment::new_enclosing(
                    environment,
                ))))?;
                Ok(())
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                if condition.evaluate(environment.clone())?.is_truthy() {
                    then_branch.evaluate(environment)?;
                } else if let Some(else_statement) = else_branch {
                    else_statement.evaluate(environment)?;
                }

                Ok(())
            }
            Stmt::While { condition, body } => {
                while condition.evaluate(environment.clone())?.is_truthy() {
                    body.evaluate(environment.clone())?;
                }

                Ok(())
            }
            Stmt::Function { name, params, body } => {
                // let function = LoxValue::Callable(LoxCallable::Function(FunctionStmt { name, params, body }))

                todo!()
            }
        }
    }
}

impl Evaluatable<()> for Vec<Stmt> {
    fn evaluate(&self, environment: Rc<RefCell<Environment>>) -> Result<(), RuntimeError> {
        for statement in self {
            statement.evaluate(environment.clone())?;
        }

        Ok(())
    }
}

impl Evaluatable<Rc<LoxValue>> for Expr {
    fn evaluate(
        &self,
        environment: Rc<RefCell<Environment>>,
    ) -> Result<Rc<LoxValue>, RuntimeError> {
        match self {
            Expr::Literal { value } => Ok(Rc::new(value.into())),
            Expr::Grouping { expression } => expression.evaluate(environment),
            Expr::Unary { operator, right } => {
                let right = right.evaluate(environment)?;
                let right = right.as_ref();

                match operator.token_type {
                    TokenType::Minus => match right {
                        LoxValue::Number(num) => Ok(Rc::new(LoxValue::Number(-num))),
                        _ => Err(RuntimeError::new(
                            operator.to_owned(),
                            "Cannot negate non numeric value".to_string(),
                        )),
                    },
                    TokenType::Bang => Ok(Rc::new(LoxValue::Boolean(!right.is_truthy()))),
                    _ => panic!("Invalid unary expression in AST"),
                }
            }
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let left = left.evaluate(environment.clone())?;
                let left_value = left.as_ref();

                let right = right.evaluate(environment)?;
                let right_value = right.as_ref();

                match operator.token_type {
                    // Arithmetic operations
                    TokenType::Minus => match (left_value, right_value) {
                        (LoxValue::Number(left_num), LoxValue::Number(right_num)) => {
                            Ok(Rc::new(LoxValue::Number(left_num - right_num)))
                        }
                        _ => Err(RuntimeError::new(
                            operator.to_owned(),
                            "Operands must be numbers.".to_string(),
                        )),
                    },
                    TokenType::Slash => match (left_value, right_value) {
                        (LoxValue::Number(left_num), LoxValue::Number(right_num)) => {
                            if right_num == &0_f64 {
                                return Err(RuntimeError::new(
                                    operator.to_owned(),
                                    "Cannot divide by 0.".to_string(),
                                ));
                            }
                            Ok(Rc::new(LoxValue::Number(left_num / right_num)))
                        }
                        _ => Err(RuntimeError::new(
                            operator.to_owned(),
                            "Operands must be numbers.".to_string(),
                        )),
                    },
                    TokenType::Star => match (left_value, right_value) {
                        (LoxValue::Number(left_num), LoxValue::Number(right_num)) => {
                            Ok(Rc::new(LoxValue::Number(left_num * right_num)))
                        }
                        _ => Err(RuntimeError::new(
                            operator.to_owned(),
                            "Operands must be numbers.".to_string(),
                        )),
                    },
                    TokenType::Plus => {
                        match (left_value, right_value) {
                            (LoxValue::Number(left_num), LoxValue::Number(right_num)) => {
                                Ok(Rc::new(LoxValue::Number(left_num + right_num)))
                            }

                            // If either one of the values is a str, we cast the other one to a string
                            (LoxValue::String(left_str), right_val) => Ok(Rc::new(
                                LoxValue::String(format!("{}{}", left_str, right_val.stringify())),
                            )),
                            (left_val, LoxValue::String(right_str)) => Ok(Rc::new(
                                LoxValue::String(format!("{}{}", left_val.stringify(), right_str)),
                            )),

                            _ => Err(RuntimeError::new(
                                operator.to_owned(),
                                "Operands must be two numbers or two strings.".to_string(),
                            )),
                        }
                    }

                    // Comparison operations
                    TokenType::Greater => match (left_value, right_value) {
                        (LoxValue::Number(left_num), LoxValue::Number(right_num)) => {
                            Ok(Rc::new(LoxValue::Boolean(left_num > right_num)))
                        }
                        _ => Err(RuntimeError::new(
                            operator.to_owned(),
                            "Operands must be numbers.".to_string(),
                        )),
                    },
                    TokenType::GreaterEqual => match (left_value, right_value) {
                        (LoxValue::Number(left_num), LoxValue::Number(right_num)) => {
                            Ok(Rc::new(LoxValue::Boolean(left_num >= right_num)))
                        }
                        _ => Err(RuntimeError::new(
                            operator.to_owned(),
                            "Operands must be numbers.".to_string(),
                        )),
                    },
                    TokenType::Less => match (left_value, right_value) {
                        (LoxValue::Number(left_num), LoxValue::Number(right_num)) => {
                            Ok(Rc::new(LoxValue::Boolean(left_num < right_num)))
                        }
                        _ => Err(RuntimeError::new(
                            operator.to_owned(),
                            "Operands must be numbers.".to_string(),
                        )),
                    },
                    TokenType::LessEqual => match (left_value, right_value) {
                        (LoxValue::Number(left_num), LoxValue::Number(right_num)) => {
                            Ok(Rc::new(LoxValue::Boolean(left_num <= right_num)))
                        }
                        _ => Err(RuntimeError::new(
                            operator.to_owned(),
                            "Operands must be numbers.".to_string(),
                        )),
                    },

                    // Equality operations
                    TokenType::BangEqual => {
                        Ok(Rc::new(LoxValue::Boolean(left_value != right_value)))
                    }
                    TokenType::EqualEqual => {
                        Ok(Rc::new(LoxValue::Boolean(left_value == right_value)))
                    }
                    _ => Err(RuntimeError::new(
                        operator.to_owned(),
                        "Invalid binary operator.".to_string(),
                    )),
                }
            }
            Expr::Variable { name } => environment.borrow().get(name),
            Expr::Assign { name, value } => {
                let value = value.evaluate(environment.clone())?;
                environment.borrow_mut().assign(name, value.clone())?;
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
            Expr::Logical {
                left,
                operator,
                right,
            } => {
                let left = left.evaluate(environment.clone())?;

                match operator.token_type {
                    TokenType::Or => {
                        if left.is_truthy() {
                            return Ok(left);
                        }
                    }
                    TokenType::And => {
                        if !left.is_truthy() {
                            return Ok(left);
                        }
                    }
                    _ => {
                        return Err(RuntimeError::new(
                            operator.to_owned(),
                            "Invalid Logical operator.".to_string(),
                        ))
                    }
                }

                right.evaluate(environment)
            }
            Expr::Call {
                callee,
                closing_paren,
                arguments,
            } => {
                let callee = callee.evaluate(environment.clone())?;

                let mut evaluated_args = vec![];
                for arg in arguments {
                    evaluated_args.push(arg.evaluate(environment.clone())?);
                }

                let function = match callee.as_ref() {
                    LoxValue::Callable(callable) => callable,
                    _ => {
                        return Err(RuntimeError::new(
                            closing_paren.to_owned(),
                            "Can only call functions and classes.".to_string(),
                        ))
                    }
                };

                function.call(evaluated_args, closing_paren, environment)
            }
        }
    }
}
