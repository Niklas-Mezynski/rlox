use std::{collections::HashMap, rc::Rc};

use crate::{
    interpreter::{LoxValue, RuntimeError, RuntimeEvent, Stringifyable},
    lox_callable::LoxClass,
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

    pub fn get(&self, name: &Token) -> Result<Rc<LoxValue>, RuntimeEvent> {
        if self.fields.contains_key(&name.lexeme) {
            return Ok(self
                .fields
                .get(&name.lexeme)
                .expect("Value must be present, key was checked")
                .clone());
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
