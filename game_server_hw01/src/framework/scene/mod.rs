pub mod game_scene;

pub use game_scene::GameScene;

use super::{
    model::Model, 
    object::Object,
    camera::Camera,
    color,
};


pub trait Scene {
    fn init(&mut self, device: &wgpu::Device);

    fn handle_event(&mut self, event: &winit::event::WindowEvent) -> bool;

    fn update(&mut self);

    fn camera(&self) -> &Camera;
    fn models(&self) -> &Vec<Model>;
    fn objects(&self) -> &Vec<Object>;
    fn background_color(&self) -> color::Color;
}