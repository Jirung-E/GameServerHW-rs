use super::model::*;


pub struct Object {
    pub model: Model,
    pub position: cgmath::Vector3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
    // pub scale: f32,
}
