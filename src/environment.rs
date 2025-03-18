use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    interpreter::{LoxValue, RuntimeError, RuntimeEvent},
    token::Token,
};

#[derive(Debug)]
pub struct Environment {
    values: HashMap<String, Rc<LoxValue>>,
    enclosing: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Environment {
        Environment {
            values: HashMap::new(),
            enclosing: None,
        }
    }

    pub fn new_enclosing(enclosing: Rc<RefCell<Environment>>) -> Environment {
        Environment {
            values: HashMap::new(),
            enclosing: Some(enclosing),
        }
    }

    pub fn define(&mut self, name: String, value: Rc<LoxValue>) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &Token) -> Result<Rc<LoxValue>, RuntimeEvent> {
        if self.values.contains_key(&name.lexeme) {
            return Ok(self
                .values
                .get(&name.lexeme)
                .expect("Value must be present, key was checked")
                // For now we can just clone with very minimal overhead. The value will be copied when used in the interpreter anyways.
                // In case we have to store larger objects we can consider storing Rc<...>s in the HashMap
                .clone());
        }

        if let Some(enclosing) = self.enclosing.as_ref() {
            return enclosing.borrow().get(name);
        }

        Err(RuntimeEvent::Error(RuntimeError {
            token: name.clone(),
            message: format!("Undefined variable '{}'.", name.lexeme),
        }))
    }

    pub fn assign(&mut self, name: &Token, value: Rc<LoxValue>) -> Result<(), RuntimeEvent> {
        if self.values.contains_key(&name.lexeme) {
            *self
                .values
                .get_mut(&name.lexeme)
                .expect("Value must be present, key was checked") = value;
            return Ok(());
        }

        if let Some(enclosing) = self.enclosing.as_ref() {
            return enclosing.borrow_mut().assign(name, value);
        }

        Err(RuntimeEvent::Error(RuntimeError {
            token: name.clone(),
            message: format!("Undefined variable '{}'.", name.lexeme),
        }))
    }
}
