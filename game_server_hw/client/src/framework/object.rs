use super::model::*;
use super::transform::*;

use std::{
    rc::Weak, 
    cell::RefCell,
};

pub struct Object {
    pub model: Weak<RefCell<Model>>, 
    pub transform: Transform,
}


impl Object {
    pub fn new() -> Self {
        Self {
            model: Weak::new(), 
            transform: Transform::default(),
        }
    }

    pub fn set_model(&mut self, model: Weak<RefCell<Model>>) {
        self.model = model;
    }
}
