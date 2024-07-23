use super::model::*;
use super::transform::*;


pub struct Object {
    pub model: Option<*mut Model>,
    pub transform: Transform,
}


impl Object {
    pub fn new() -> Self {
        Self {
            model: None,
            transform: Transform::default(),
        }
    }

    pub fn set_model(&mut self, model: &mut Model) {
        model.add_instance(&self.transform);
        self.model = Some(model);
    }
}
