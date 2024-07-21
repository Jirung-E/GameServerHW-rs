pub mod game_scene;

pub use game_scene::GameScene;

use super::{
    model::Model, 
    object::Object,
    color,
};


pub trait Scene {
    fn init(&mut self, device: &wgpu::Device);

    fn handle_event(&mut self, event: &winit::event::WindowEvent) -> bool;

    fn update(&mut self);

    fn view_proj(&self) -> cgmath::Matrix4<f32>;
    fn models(&self) -> &Vec<Model>;
    fn objects(&self) -> &Vec<Object>;
    fn background_color(&self) -> color::Color;
}