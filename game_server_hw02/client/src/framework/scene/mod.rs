pub mod game_scene;

pub use game_scene::GameScene;

use super::{
    model::Model, 
    object::Object,
    color,
};

use std::{
    rc::Rc, 
    cell::RefCell, 
    iter::Iterator
};


pub trait Scene {
    fn init(&mut self, device: &wgpu::Device);

    fn handle_event(&mut self, event: &winit::event::WindowEvent) -> bool;

    fn update(&mut self);

    fn view_proj(&self) -> cgmath::Matrix4<f32>;
    fn background_color(&self) -> color::Color;

    fn models(&self) -> impl Iterator<Item = &Rc<RefCell<Model>>>;
    fn objects(&self) -> impl Iterator<Item = &Rc<RefCell<Object>>>;
}