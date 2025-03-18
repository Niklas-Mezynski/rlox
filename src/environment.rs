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

    pub fn get_at(
        &self,
        distance: Option<usize>,
        name: &Token,
    ) -> Result<Rc<LoxValue>, RuntimeEvent> {
        match self.ancestor(distance) {
            Some(env) => env.borrow().get_this(name),
            None => self.get_this(name),
        }
    }

    fn get_this(&self, name: &Token) -> Result<Rc<LoxValue>, RuntimeEvent> {
        if self.values.contains_key(&name.lexeme) {
            return Ok(self
                .values
                .get(&name.lexeme)
                .expect("Value must be present, key was checked")
                .clone());
        }

        Err(RuntimeEvent::Error(RuntimeError {
            token: name.clone(),
            message: format!("Undefined variable '{}'.", name.lexeme),
        }))
    }

    fn ancestor(&self, distance: Option<usize>) -> Option<Rc<RefCell<Environment>>> {
        match distance {
            Some(0) => None, // Current environment
            Some(n) => {
                let mut current = self.enclosing.clone();
                for _ in 1..n {
                    current = current
                        .expect("Environment at depth must exist as it was checked in resolver")
                        .borrow()
                        .enclosing
                        .clone();
                }
                current
            }
            None => {
                // Find the global (outermost) environment
                if let Some(enclosing) = &self.enclosing {
                    let mut current = enclosing.clone();

                    loop {
                        let next_env = {
                            let env_ref = current.borrow();
                            env_ref.enclosing.clone()
                        };

                        match next_env {
                            Some(env) => current = env,
                            None => break,
                        }
                    }

                    Some(current)
                } else {
                    None // This is already the global environment
                }
            }
        }
    }

    pub fn assign_at(
        &mut self,
        distance: Option<usize>,
        name: &Token,
        value: Rc<LoxValue>,
    ) -> Result<(), RuntimeEvent> {
        match self.ancestor(distance) {
            Some(env) => env.borrow_mut().assign_this(name, value),
            None => self.assign_this(name, value),
        }
    }

    fn assign_this(&mut self, name: &Token, value: Rc<LoxValue>) -> Result<(), RuntimeEvent> {
        if self.values.contains_key(&name.lexeme) {
            *self
                .values
                .get_mut(&name.lexeme)
                .expect("Value must be present, key was checked") = value;
            return Ok(());
        }

        Err(RuntimeEvent::Error(RuntimeError {
            token: name.clone(),
            message: format!("Undefined variable '{}'.", name.lexeme),
        }))
    }
}
