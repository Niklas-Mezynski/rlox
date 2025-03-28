use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    fmt::Debug,
    rc::Rc,
};

use crate::{
    environment::Environment,
    error,
    expr::Expr,
    lox_callable::{FunctionStmt, LoxCallable},
    lox_class::LoxClass,
    lox_instance::LoxInstance,
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
    Instance(Rc<RefCell<LoxInstance>>),
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

pub enum RuntimeEvent {
    Error(RuntimeError),
    Return(Rc<LoxValue>),
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

trait MyPartialEq {
    fn eq(&self, other: &Self) -> bool;
    fn ne(&self, other: &Self) -> bool;
}

impl MyPartialEq for Rc<LoxValue> {
    fn eq(&self, other: &Self) -> bool {
        match (self.as_ref(), other.as_ref()) {
            (LoxValue::String(l0), LoxValue::String(r0)) => l0 == r0,
            (LoxValue::Number(l0), LoxValue::Number(r0)) => l0 == r0,
            (LoxValue::Boolean(l0), LoxValue::Boolean(r0)) => l0 == r0,
            (LoxValue::Nil, LoxValue::Nil) => true,
            // For other values, compare by reference
            _ => Rc::ptr_eq(self, other),
        }
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

impl Stringifyable for LoxValue {
    fn stringify(&self) -> String {
        match self {
            LoxValue::Nil => String::from("nil"),
            LoxValue::Boolean(value) => value.to_string(),
            LoxValue::Number(value) => value.to_string(),
            LoxValue::String(value) => value.clone(),
            LoxValue::Callable(value) => value.stringify(),
            LoxValue::Instance(value) => value.borrow().stringify(),
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
                match err {
                    RuntimeEvent::Error(err) => {
                        error::runtime_error(err);
                        return;
                    }
                    _ => panic!("Unhandled return statement"),
                }
            }
        }
    }
}

pub trait Evaluatable<T> {
    fn evaluate(&self, environment: Rc<RefCell<Environment>>) -> Result<T, RuntimeEvent>;
}

impl Evaluatable<()> for Stmt {
    fn evaluate(&self, environment: Rc<RefCell<Environment>>) -> Result<(), RuntimeEvent> {
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
                let function = LoxValue::Callable(LoxCallable::new_function(
                    Rc::new(FunctionStmt {
                        name: name.clone(),
                        params: params.clone(),
                        body: body.clone(),
                    }),
                    environment.clone(),
                    false,
                ));

                environment
                    .borrow_mut()
                    .define(name.lexeme.to_owned(), Rc::new(function));

                Ok(())
            }
            Stmt::Return { keyword: _, value } => {
                let value = match value {
                    Some(v) => v.evaluate(environment)?,
                    None => Rc::new(LoxValue::Nil),
                };

                Err(RuntimeEvent::Return(value))
            }
            Stmt::Class {
                name,
                superclass,
                methods,
            } => {
                let superclass = if let Some(superclass_expr) = superclass {
                    let superclass_value = superclass_expr.evaluate(environment.clone())?;

                    if !matches!(
                        superclass_value.as_ref(),
                        LoxValue::Callable(LoxCallable::Class { .. })
                    ) {
                        let error_token = match superclass_expr {
                            Expr::Variable { name, .. } => name.clone(),
                            _ => unreachable!(),
                        };

                        return Err(RuntimeEvent::Error(RuntimeError::new(
                            error_token,
                            "Superclass must be a class.".to_string(),
                        )));
                    }

                    Some(superclass_value)
                } else {
                    None
                };

                {
                    // Bind the own classname (in the beginning this is Nil, as class is not fully initialized)
                    let mut env_mut = environment.borrow_mut();
                    env_mut.define(name.lexeme.to_string(), Rc::new(LoxValue::Nil));
                }

                let mut parent_environment = None;

                let mut environment = if let Some(superclass) = &superclass {
                    parent_environment = Some(environment.clone());
                    let super_env = Rc::new(RefCell::new(Environment::new_enclosing(environment)));
                    super_env
                        .borrow_mut()
                        .define("super".to_string(), superclass.clone());
                    super_env
                } else {
                    environment
                };

                let method_map: HashMap<String, Rc<LoxValue>> = methods
                    .iter()
                    .map(|method| match method {
                        Stmt::Function { name, params, body } => (
                            name.lexeme.to_string(),
                            Rc::new(LoxValue::Callable(LoxCallable::new_function(
                                Rc::new(FunctionStmt {
                                    name: name.clone(),
                                    params: params.clone(),
                                    body: body.clone(),
                                }),
                                // This is either the current environment, or the one which is bound with "super"
                                environment.clone(),
                                name.lexeme == "init",
                            ))),
                        ),
                        _ => panic!("Class can only contain methods."),
                    })
                    .collect();

                let class = LoxValue::Callable(LoxCallable::Class {
                    class: Rc::new(LoxClass::new(
                        name.lexeme.to_string(),
                        superclass,
                        method_map,
                    )),
                });

                if let Some(old_environment) = parent_environment {
                    environment = old_environment;
                }

                environment.borrow_mut().assign(name, Rc::new(class))?;

                Ok(())
            }
        }
    }
}

impl Evaluatable<()> for Vec<Stmt> {
    fn evaluate(&self, environment: Rc<RefCell<Environment>>) -> Result<(), RuntimeEvent> {
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
    ) -> Result<Rc<LoxValue>, RuntimeEvent> {
        match self {
            Expr::Literal { value } => Ok(Rc::new(value.into())),
            Expr::Grouping { expression } => expression.evaluate(environment),
            Expr::Unary { operator, right } => {
                let right = right.evaluate(environment)?;
                let right = right.as_ref();

                match operator.token_type {
                    TokenType::Minus => match right {
                        LoxValue::Number(num) => Ok(Rc::new(LoxValue::Number(-num))),
                        _ => Err(RuntimeEvent::Error(RuntimeError::new(
                            operator.to_owned(),
                            "Cannot negate non numeric value".to_string(),
                        ))),
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
                        _ => Err(RuntimeEvent::Error(RuntimeError::new(
                            operator.to_owned(),
                            "Operands must be numbers.".to_string(),
                        ))),
                    },
                    TokenType::Slash => match (left_value, right_value) {
                        (LoxValue::Number(left_num), LoxValue::Number(right_num)) => {
                            if right_num == &0_f64 {
                                return Err(RuntimeEvent::Error(RuntimeError::new(
                                    operator.to_owned(),
                                    "Cannot divide by 0.".to_string(),
                                )));
                            }
                            Ok(Rc::new(LoxValue::Number(left_num / right_num)))
                        }
                        _ => Err(RuntimeEvent::Error(RuntimeError::new(
                            operator.to_owned(),
                            "Operands must be numbers.".to_string(),
                        ))),
                    },
                    TokenType::Star => match (left_value, right_value) {
                        (LoxValue::Number(left_num), LoxValue::Number(right_num)) => {
                            Ok(Rc::new(LoxValue::Number(left_num * right_num)))
                        }
                        _ => Err(RuntimeEvent::Error(RuntimeError::new(
                            operator.to_owned(),
                            "Operands must be numbers.".to_string(),
                        ))),
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

                            _ => Err(RuntimeEvent::Error(RuntimeError::new(
                                operator.to_owned(),
                                "Operands must be two numbers or two strings.".to_string(),
                            ))),
                        }
                    }

                    // Comparison operations
                    TokenType::Greater => match (left_value, right_value) {
                        (LoxValue::Number(left_num), LoxValue::Number(right_num)) => {
                            Ok(Rc::new(LoxValue::Boolean(left_num > right_num)))
                        }
                        _ => Err(RuntimeEvent::Error(RuntimeError::new(
                            operator.to_owned(),
                            "Operands must be numbers.".to_string(),
                        ))),
                    },
                    TokenType::GreaterEqual => match (left_value, right_value) {
                        (LoxValue::Number(left_num), LoxValue::Number(right_num)) => {
                            Ok(Rc::new(LoxValue::Boolean(left_num >= right_num)))
                        }
                        _ => Err(RuntimeEvent::Error(RuntimeError::new(
                            operator.to_owned(),
                            "Operands must be numbers.".to_string(),
                        ))),
                    },
                    TokenType::Less => match (left_value, right_value) {
                        (LoxValue::Number(left_num), LoxValue::Number(right_num)) => {
                            Ok(Rc::new(LoxValue::Boolean(left_num < right_num)))
                        }
                        _ => Err(RuntimeEvent::Error(RuntimeError::new(
                            operator.to_owned(),
                            "Operands must be numbers.".to_string(),
                        ))),
                    },
                    TokenType::LessEqual => match (left_value, right_value) {
                        (LoxValue::Number(left_num), LoxValue::Number(right_num)) => {
                            Ok(Rc::new(LoxValue::Boolean(left_num <= right_num)))
                        }
                        _ => Err(RuntimeEvent::Error(RuntimeError::new(
                            operator.to_owned(),
                            "Operands must be numbers.".to_string(),
                        ))),
                    },

                    // Equality operations
                    TokenType::BangEqual => Ok(Rc::new(LoxValue::Boolean(left.ne(&right)))),
                    TokenType::EqualEqual => Ok(Rc::new(LoxValue::Boolean(left.eq(&right)))),
                    _ => Err(RuntimeEvent::Error(RuntimeError::new(
                        operator.to_owned(),
                        "Invalid binary operator.".to_string(),
                    ))),
                }
            }
            Expr::Variable { name, depth } => environment.borrow().get_at(*depth, name),
            Expr::Assign { name, value, depth } => {
                let value = value.evaluate(environment.clone())?;
                environment
                    .borrow_mut()
                    .assign_at(*depth, name, value.clone())?;
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
                        return Err(RuntimeEvent::Error(RuntimeError::new(
                            operator.to_owned(),
                            "Invalid Logical operator.".to_string(),
                        )))
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

                let mut evaluated_args = VecDeque::new();
                for arg in arguments {
                    evaluated_args.push_back(arg.evaluate(environment.clone())?);
                }

                let function = match callee.as_ref() {
                    LoxValue::Callable(callable) => callable,
                    _ => {
                        return Err(RuntimeEvent::Error(RuntimeError::new(
                            closing_paren.to_owned(),
                            "Can only call functions and classes.".to_string(),
                        )))
                    }
                };

                function.call(evaluated_args, closing_paren)
            }
            Expr::Get { object, name } => {
                let object = object.evaluate(environment)?;

                match object.as_ref() {
                    LoxValue::Instance(lox_instance) => {
                        LoxInstance::get(lox_instance.clone(), name)
                    }
                    _ => Err(RuntimeEvent::Error(RuntimeError::new(
                        name.to_owned(),
                        "Only instances have properties.".to_string(),
                    ))),
                }
            }
            Expr::Set {
                object,
                name,
                value,
            } => {
                let object = object.evaluate(environment.clone())?;

                match object.as_ref() {
                    LoxValue::Instance(lox_instance) => {
                        let value = value.evaluate(environment)?;
                        lox_instance.borrow_mut().set(name, value.clone())?;
                        Ok(value)
                    }
                    _ => Err(RuntimeEvent::Error(RuntimeError::new(
                        name.to_owned(),
                        "Only instances have properties.".to_string(),
                    ))),
                }
            }
            Expr::This { keyword, depth } => environment.borrow().get_at(Some(*depth), keyword),
            Expr::Super {
                keyword,
                method,
                depth,
            } => {
                // First get the superclass value and extend its lifetime
                let superclass_value = environment.borrow().get_at(Some(*depth), keyword)?;
                let superclass = match superclass_value.as_ref() {
                    LoxValue::Callable(LoxCallable::Class { class }) => class,
                    _ => panic!("Superclass must be LoxClass"),
                };

                let this_value = environment.borrow().get_at(
                    Some(*depth - 1),
                    &Token::new(TokenType::This, "this".to_string(), keyword.line),
                )?;

                let object = match this_value.as_ref() {
                    LoxValue::Instance(instance) => instance,
                    _ => panic!("'this' while evaluating super, must be LoxInstance"),
                };

                let method_value = superclass.find_method(&method.lexeme);

                match method_value {
                    Some(method) => Ok(method.bind(object.clone())),
                    None => Err(RuntimeEvent::Error(RuntimeError::new(
                        method.to_owned(),
                        format!("Undefined property '{}'.", method.lexeme),
                    ))),
                }
            }
        }
    }
}
