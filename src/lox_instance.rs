use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    environment::Environment,
    interpreter::{LoxValue, RuntimeError, RuntimeEvent, Stringifyable},
    lox_callable::LoxCallable,
    lox_class::LoxClass,
    token::Token,
};

#[derive(Debug)]
pub struct LoxInstance {
    klass: Rc<LoxClass>,
    fields: HashMap<String, Rc<LoxValue>>,
}

impl Stringifyable for LoxInstance {
    fn stringify(&self) -> String {
        format!("{} instance", self.klass.name)
    }
}

impl LoxInstance {
    pub fn new(klass: Rc<LoxClass>) -> LoxInstance {
        LoxInstance {
            klass,
            fields: HashMap::new(),
        }
    }

    pub fn get(this: Rc<RefCell<LoxInstance>>, name: &Token) -> Result<Rc<LoxValue>, RuntimeEvent> {
        if this.borrow().fields.contains_key(&name.lexeme) {
            return Ok(this
                .borrow()
                .fields
                .get(&name.lexeme)
                .expect("Value must be present, key was checked")
                .clone());
        }

        if let Some(method) = this.borrow().klass.find_method(&name.lexeme) {
            return Ok(method.bind(this.clone()));
        }

        Err(RuntimeEvent::Error(RuntimeError {
            token: name.clone(),
            message: format!("Undefined property '{}'.", name.lexeme),
        }))
    }

    pub fn set(&mut self, name: &Token, value: Rc<LoxValue>) -> Result<(), RuntimeEvent> {
        self.fields.insert(name.lexeme.to_string(), value);
        Ok(())
    }
}

impl LoxValue {
    pub fn bind(&self, instance: Rc<RefCell<LoxInstance>>) -> Rc<LoxValue> {
        match self {
            LoxValue::Callable(callable) => match callable {
                LoxCallable::Function {
                    declaration,
                    closure,
                    is_initializer,
                } => {
                    let mut environment = Environment::new_enclosing(closure.clone());
                    environment.define("this".to_string(), Rc::new(LoxValue::Instance(instance)));
                    Rc::new(LoxValue::Callable(LoxCallable::new_function(
                        declaration.clone(),
                        Rc::new(RefCell::new(environment)),
                        *is_initializer,
                    )))
                }
                _ => todo!(),
            },
            _ => todo!(),
        }
    }
}
