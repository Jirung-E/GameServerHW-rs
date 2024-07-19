use winit::event::{
    ElementState, KeyEvent, WindowEvent
};
use winit::keyboard::{KeyCode, PhysicalKey};
use cgmath::prelude::*;

use super::super::{
    camera::Camera,
    object::Object,
    model::Model,
    SCREEN_WIDTH, SCREEN_HEIGHT,
};
use super::Scene;
use super::color::Color;




struct CameraController {
    speed: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
}

impl CameraController {
    fn new(speed: f32) -> Self {
        Self {
            speed,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
        }
    }

    fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state,
                        physical_key: PhysicalKey::Code(keycode),
                        ..
                    },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {
                    KeyCode::ArrowUp => {
                        self.is_forward_pressed = is_pressed;
                        true
                    }
                    KeyCode::ArrowLeft => {
                        self.is_left_pressed = is_pressed;
                        true
                    }
                    KeyCode::ArrowDown => {
                        self.is_backward_pressed = is_pressed;
                        true
                    }
                    KeyCode::ArrowRight => {
                        self.is_right_pressed = is_pressed;
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    fn update_camera(&self, camera: &mut Camera) {
        use cgmath::InnerSpace;
        let forward = camera.target - camera.eye;
        let forward_norm = forward.normalize();
        let forward_mag = forward.magnitude();

        // Prevents glitching when the camera gets too close to the
        // center of the scene.
        if self.is_forward_pressed && forward_mag > self.speed {
            camera.eye += forward_norm * self.speed;
        }
        if self.is_backward_pressed {
            camera.eye -= forward_norm * self.speed;
        }

        let right = forward_norm.cross(camera.up);

        // Redo radius calc in case the forward/backward is pressed.
        let forward = camera.target - camera.eye;
        let forward_mag = forward.magnitude();

        if self.is_right_pressed {
            // Rescale the distance between the target and the eye so 
            // that it doesn't change. The eye, therefore, still 
            // lies on the circle made by the target and eye.
            camera.eye = camera.target - (forward + right * self.speed).normalize() * forward_mag;
        }
        if self.is_left_pressed {
            camera.eye = camera.target - (forward - right * self.speed).normalize() * forward_mag;
        }
    }
}




pub struct GameScene {
    camera: Camera,
    camera_controller: CameraController,

    models: Vec<Model>,
    objects: Vec<Object>,

    background_color: Color,
}

impl GameScene {
    const ROTATION_SPEED: f32 = 2.0 * std::f32::consts::PI / 60.0 / 100.0;

    pub fn new() -> Self {
        let camera = Camera {
            eye: (0.0, 1.0, 2.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(), 
            aspect: SCREEN_WIDTH as f32 / SCREEN_HEIGHT as f32,
            fovy: 60.0,
            znear: 0.1,
            zfar: 100.0,
        };

        Self {
            camera,
            camera_controller: CameraController::new(0.01),

            models: Vec::new(),
            objects: Vec::new(),

            background_color: Color::BLACK,
        }
    }
}

impl Scene for GameScene {
    fn init(&mut self, device: &wgpu::Device) {
        use futures::executor::block_on;
        
        block_on(async {
            self.models = vec![
                Model::load("cube.obj", device, 0.5, Color::LIGHT_GRAY).await.unwrap(),
                Model::load("cube.obj", device, 0.5, Color::DARK_GRAY).await.unwrap(),
                Model::load("knight.obj", device, 0.8, Color::WHITE).await.unwrap(),
                Model::load("knight.obj", device, 0.8, Color::BLACK).await.unwrap(),
            ];
        });

        
        self.objects = (0..64)
            .map(|_| Object::new())
            .collect::<Vec<_>>();

        for (i, object) in self.objects.iter_mut().enumerate() {
            let x = i % 8;
            let z = i / 8;
            object.transform.position = cgmath::Vector3::new(
                x as f32,
                -0.5,
                z as f32
            );
            object.set_model(&mut self.models[(x+z) & 1]);
        }

        self.objects.push(Object::new());
        self.objects.last_mut().unwrap().set_model(&mut self.models[2]);
        self.objects.last_mut().unwrap().transform.position = cgmath::Vector3::new(
            3.0,
            0.1,
            2.0,
        );
    }

    fn handle_event(&mut self, event: &WindowEvent) -> bool {
        if self.camera_controller.process_events(event) {
            return true;
        }

        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state: ElementState::Pressed,
                        physical_key: PhysicalKey::Code(keycode),
                        repeat: false,
                        ..
                    },
                ..
            } => {
                match keycode {
                    KeyCode::KeyW => {
                        self.objects.last_mut().unwrap().transform.position.z -= 1.0;
                        true
                    }
                    KeyCode::KeyA => {
                        self.objects.last_mut().unwrap().transform.position.x -= 1.0;
                        true
                    }
                    KeyCode::KeyS => {
                        self.objects.last_mut().unwrap().transform.position.z += 1.0;
                        true
                    }
                    KeyCode::KeyD => {
                        self.objects.last_mut().unwrap().transform.position.x += 1.0;
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    fn update(&mut self) {
        self.camera_controller.update_camera(&mut self.camera);

        for object in self.objects[64..].iter_mut() {
            object.transform.rotation = cgmath::Quaternion::from_angle_y(
                cgmath::Rad(GameScene::ROTATION_SPEED)
            ) * object.transform.rotation;
        }
    }


    fn camera(&self) -> &Camera {
        &self.camera
    }

    fn models(&self) -> &Vec<Model> {
        &self.models
    }

    fn objects(&self) -> &Vec<Object> {
        &self.objects
    }

    fn background_color(&self) -> Color {
        self.background_color
    }
}