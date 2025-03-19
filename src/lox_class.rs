use std::{collections::HashMap, rc::Rc};

use crate::interpreter::LoxValue;

#[derive(Debug)]
pub struct LoxClass {
    pub name: String,
    methods: HashMap<String, Rc<LoxValue>>,
}

impl LoxClass {
    pub fn new(name: String, methods: HashMap<String, Rc<LoxValue>>) -> LoxClass {
        LoxClass { name, methods }
    }

    pub fn find_method(&self, name: &str) -> Option<Rc<LoxValue>> {
        self.methods.get(name).map(Rc::clone)
    }
}
