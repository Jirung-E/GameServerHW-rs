use winit::event::{
    ElementState, KeyEvent, WindowEvent
};
use winit::keyboard::{KeyCode, PhysicalKey};

use super::super::{
    camera::{Camera, CameraComponent, DefaultCamera},
    object::Object,
    model::Model,
    SCREEN_WIDTH, SCREEN_HEIGHT,
};
use super::Scene;
use super::color::Color;


pub struct GameScene {
    camera: DefaultCamera,
    camera_offset: cgmath::Vector3<f32>,

    background_color: Color,

    models: Vec<Model>,
    objects: Vec<Object>,

    player: *mut Object,
}

impl GameScene {
    pub fn new() -> Self {
        let camera = DefaultCamera::from(CameraComponent {
            eye: cgmath::Point3::new(0.0, 1.0, 2.0),
            target: cgmath::Point3::new(0.0, 0.0, 0.0),
            up: cgmath::Vector3::new(0.0, 1.0, 0.0),
            aspect: SCREEN_WIDTH as f32 / SCREEN_HEIGHT as f32,
            fovy: 60.0,
            znear: 0.1,
            zfar: 100.0,
        });

        Self {
            camera,
            camera_offset: cgmath::Vector3::new(0.0, 2.0, 4.0),

            background_color: Color::BLACK,

            models: Vec::new(),
            objects: Vec::new(),

            player: std::ptr::null_mut(),
        }
    }

    fn update_camera(&mut self) {
        unsafe {
            let p = (*self.player).transform.position;

            let point = cgmath::Point3::new(p.x, p.y, p.z);

            self.camera.component.target = point;
            self.camera.component.eye = point + self.camera_offset;
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

        self.player = self.objects.last_mut().unwrap();
    }

    fn handle_event(&mut self, event: &WindowEvent) -> bool {
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
        self.update_camera();
    }


    fn view_proj(&self) -> cgmath::Matrix4<f32> {
        self.camera.build_view_projection_matrix()
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
