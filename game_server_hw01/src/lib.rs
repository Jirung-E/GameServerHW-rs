pub mod framework;




////////////////////////////////////////////////////////////////////////////////




use winit::{
    event::*,
    event_loop::{EventLoop, EventLoopWindowTarget},
    keyboard::{KeyCode, PhysicalKey},
    window::WindowBuilder,
};

use framework::state::*;


/// 이벤트루프 시작 및 윈도우 생성
#[tokio::main]
pub async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut state = State::new(&window).await;

    let _ = event_loop.run(move |event, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window().id() => if !state.input(event) {
                match event {
                    WindowEvent::CloseRequested => control_flow.exit(),

                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    },

                    WindowEvent::KeyboardInput { 
                        event: key_event, 
                        .. 
                    } => handle_keyboard_input(&key_event, &control_flow),

                    WindowEvent::CursorMoved { position, .. } => {
                        state.background_color = wgpu::Color {
                            r: position.x as f64 / state.size.width as f64,
                            g: position.y as f64 / state.size.height as f64,
                            b: 1.0,
                            a: 1.0,
                        };
                    }

                    WindowEvent::RedrawRequested => {
                        state.window().request_redraw();
                        
                        state.update();
                        match state.render() {
                            Ok(_) => {}
                            // Reconfigure the surface if lost
                            Err(
                                wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated,
                            ) => state.resize(state.size),
                            // The system is out of memory, we should probably quit
                            Err(wgpu::SurfaceError::OutOfMemory) => control_flow.exit(),
                            // All other errors (Outdated, Timeout) should be resolved by the next frame
                            Err(e) => eprintln!("{:?}", e),
                        }
                    },

                    _ => {}
                }
            },

            _ => {}
        }
    });
}


/// 키보드 입력 처리
fn handle_keyboard_input(key_event: &KeyEvent, control_flow: &EventLoopWindowTarget<()>) {
    match key_event {
        KeyEvent {
            state: ElementState::Pressed,
            physical_key: PhysicalKey::Code(key),
            ..
        } => {
            handle_key_press(key, control_flow);
        },
        
        KeyEvent {
            state: ElementState::Released,
            physical_key: PhysicalKey::Code(key),
            ..
        } => {
            handle_key_release(key, control_flow);
        },

        _ => {}
    }
}

fn handle_key_press(key: &KeyCode, _control_flow: &EventLoopWindowTarget<()>) {
    match key {
        KeyCode::KeyW => println!("Key W pressed"),
        KeyCode::KeyA => println!("Key A pressed"),
        KeyCode::KeyS => println!("Key S pressed"),
        KeyCode::KeyD => println!("Key D pressed"),
        _ => {}
    }
}

fn handle_key_release(key: &KeyCode, control_flow: &EventLoopWindowTarget<()>) {
    match key {
        KeyCode::KeyW => println!("Key W released"),
        KeyCode::KeyA => println!("Key A released"),
        KeyCode::KeyS => println!("Key S released"),
        KeyCode::KeyD => println!("Key D released"),
        KeyCode::Escape => control_flow.exit(),
        _ => {}
    }
}