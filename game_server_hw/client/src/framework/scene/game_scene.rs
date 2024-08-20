use winit::{
    event::{ElementState, KeyEvent, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};
use cgmath::{Vector2, Vector3, Point3};
use futures::executor::block_on;
use std::{
    rc::Rc, 
    cell::RefCell, 
    io::{Read, Write}, 
    net::TcpStream,
    collections::HashMap,
    iter::IntoIterator,
};
use get_addr::get_addr;
use network::*;

use super::super::{
    camera::{Camera, CameraComponent, DefaultCamera},
    object::Object,
    model::Model,
    color::Color,
    SCREEN_WIDTH, SCREEN_HEIGHT,
};
use super::Scene;


pub struct GameScene {
    camera: DefaultCamera,
    camera_offset: Vector3<f32>,

    background_color: Color,

    models: Vec<Rc<RefCell<Model>>>,
    objects: Vec<Rc<RefCell<Object>>>,
    objects_from_server: HashMap<u32, Rc<RefCell<Object>>>,

    player_id: u32,

    stream: TcpStream,
    packet_parser: PacketParser,
}

impl GameScene {
    pub async fn new() -> Self {
        let camera = DefaultCamera::from(CameraComponent {
            eye: Point3::new(0.0, 1.0, 2.0),
            target: Point3::new(0.0, 0.0, 0.0),
            up: Vector3::new(0.0, 1.0, 0.0),
            aspect: SCREEN_WIDTH as f32 / SCREEN_HEIGHT as f32,
            fovy: 60.0,
            znear: 0.1,
            zfar: 100.0,
        });

        let (ip, port) = match get_addr() {
            Ok((ip, port)) => (ip, port),
            Err(e) => {
                panic!("{}", e);
            }
        };
        // let ip = "127.0.0.1".to_string();
        // let port = 8080;
        let addr = format!("{}:{}", ip, port);
        let stream = TcpStream::connect(addr).unwrap();
        stream.set_nonblocking(true).unwrap();

        Self {
            camera,
            camera_offset: Vector3::new(0.0, 2.0, 4.0),

            background_color: Color::from_rgb(0.1, 0.3, 0.5),

            models: Vec::new(),
            objects: Vec::new(),
            objects_from_server: HashMap::new(),

            player_id: 0,

            // ip,
            // port,
            stream,
            packet_parser: PacketParser::new(),
        }
    }

    fn load_models(&mut self, device: &wgpu::Device) {
        block_on(async {
            self.models = [
                Model::load("cube.obj", device, 0.5, Color::LIGHT_GRAY).await.unwrap(),
                Model::load("cube.obj", device, 0.5, Color::DARK_GRAY).await.unwrap(),
                Model::load("pawn.obj", device, 0.8, Color::WHITE).await.unwrap(),
                Model::load("pawn.obj", device, 0.8, Color::BLACK).await.unwrap(),
            ].into_iter()
            .map(|model| Rc::new(RefCell::new(model)))
            .collect();
        });
    }

    fn build_objects(&mut self) {
        self.objects = (0..64)
            .into_iter()
            .map(|idx| {
                let object = Rc::new(RefCell::new(Object::new()));
                
                let x = idx % 8;
                let z = idx / 8;
                let model = self.models[(x+z) & 1].clone();
                model.borrow_mut().add_instance(object.clone());
                object.borrow_mut().set_model(Rc::downgrade(&model));
                object.borrow_mut().transform.position = Vector3::new(
                    x as f32, 
                    -0.5, 
                    z as f32
                );

                object
            })
            .collect();
    }

    fn player(&self) -> Option<Rc<RefCell<Object>>> {
        self.objects_from_server.get(&self.player_id).cloned()
    }

    fn update_camera(&mut self) {
        if let Some(player) = self.player(){
            let p = player.borrow().transform.position;
    
            let point = Point3::new(p.x, p.y, p.z);
    
            self.camera.component.target = point;
            self.camera.component.eye = point + self.camera_offset;
        }
    }

    fn pull_messages(&mut self) {
        let mut buf = [0; 1024];

        match self.stream.read(&mut buf) {
            Ok(0) => {
                println!("Connection closed");
            },
            Ok(n) => {
                // println!("Received: {}", msg);
                self.packet_parser.push(&buf[..n]);
            },
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // println!("Would block");
            },
            Err(_) => {
                
            }
        }
    }
    
    fn process_message(&mut self, msg: &str) {
        let msg = msg.trim().split_whitespace()
            .map(|s| s.trim())
            .collect::<Vec<&str>>();

        // println!("Received: {:?}", msg);

        if msg.len() == 0 {
            return;
        }

        match msg[0] {
            "init" => {
                if msg.len() < 2 {
                    return;
                }

                self.player_id = msg[1].parse::<u32>().unwrap();
            }

            "update" => {
                if msg.len() < 2 {
                    return;
                }

                let num_objects = msg[1].parse::<usize>().unwrap();
                let mut valid_ids: Vec<u32> = Vec::new();

                for i in 0..num_objects {
                    let idx = 2 + i * 3;
                    let id = msg[idx].parse::<u32>().unwrap();
                    let x = msg[idx+1].parse::<i32>().unwrap();
                    let z = msg[idx+2].parse::<i32>().unwrap();

                    let object = self.objects_from_server.entry(id)
                        .or_insert_with(|| {
                            let object = Rc::new(RefCell::new(Object::new()));
                            let idx = if id == self.player_id { 2 } else { 3 };
                            let model = self.models[idx].clone();
                            
                            model.borrow_mut().add_instance(object.clone());
                            object.borrow_mut().set_model(Rc::downgrade(&model));
                            object
                        });

                    let mut object = object.borrow_mut();
                    object.transform.position.x = x as f32;
                    object.transform.position.z = z as f32;
                    
                    valid_ids.push(id);
                }

                // 기존에 있던 id가 안보이면 삭제
                self.objects_from_server.retain(|k, object| {
                    let contains = valid_ids.contains(k);
                    if !contains {
                        if let Some(model) = object.borrow().model.upgrade() {
                            let mut model = model.borrow_mut();
                            model.remove_instance(object.clone());
                        }
                    }
                    contains
                });
            }
            _ => {}
        }
    }

    fn process_keyboard_input(&mut self, state: &ElementState, keycode: &KeyCode) -> bool {
        match state {
            ElementState::Pressed => {
                let mut direction = Vector2::new(0, 0);

                match keycode {
                    KeyCode::KeyW => direction.y = -1,
                    KeyCode::KeyA => direction.x = -1,
                    KeyCode::KeyS => direction.y = 1,
                    KeyCode::KeyD => direction.x = 1,
                    _ => return false,
                }
                
                println!("Move ({} {})", direction.x, direction.y);

                // println!("{}", self.stream.peer_addr().unwrap());
                let msg = format!("move {} {} {}\n", self.player_id, direction.x, direction.y);
                let packet = MessagePacket::new(0, &msg);
                self.stream.write_all(&packet.as_bytes())
                    .expect("Failed to write to stream");

                println!("Sent ok");

                true
            }
            ElementState::Released => false
        }
    }
}

impl Scene for GameScene {
    fn init(&mut self, device: &wgpu::Device) {
        self.load_models(device);
        self.build_objects();
    }

    fn handle_event(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                event: KeyEvent {
                    state,
                    physical_key: PhysicalKey::Code(keycode),
                    repeat: false,
                    ..
                },
                ..
            } => self.process_keyboard_input(state, keycode),
            _ => false,
        }
    }

    fn update(&mut self) {
        let packet = MessagePacket::new(0, "update");
        self.stream.write_all(&packet.as_bytes())
            .expect("Failed to write to stream");

        self.pull_messages();

        while let Some(packet) = self.packet_parser.pop() {
            let msg = packet.msg();
            self.process_message(&msg);
        }

        self.update_camera();
    }


    fn view_proj(&self) -> cgmath::Matrix4<f32> {
        self.camera.build_view_projection_matrix()
    }

    fn background_color(&self) -> Color {
        self.background_color
    }
    

    fn models(&self) -> impl Iterator<Item = &Rc<RefCell<Model>>> {
        self.models.iter()
    }

    fn objects(&self) -> impl Iterator<Item = &Rc<RefCell<Object>>> {
        self.objects.iter().chain(self.objects_from_server.values())
    }
}
