pub mod framework;
pub mod server;


use winit::{
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::WindowBuilder,
};
use tokio::join;

use framework::*;
use server::Server;


/// 이벤트루프 시작 및 윈도우 생성
#[tokio::main]
pub async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let (mut state, server) = join!(
        State::new(&window), 
        Server::new("127.0.0.1", 7878),
    );
    let mut server = server.unwrap();
    tokio::spawn(server.run());

    println!("start");

    let _ = event_loop.run(|event, control_flow| {
        server.receive().map(|msg| println!("{}", msg));

        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window().id() => {
                if let Some(response) = state.handle_event(event) {
                    server.send(response);
                }
                else {
                    match event {
                        WindowEvent::CloseRequested => control_flow.exit(),

                        WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size);
                        },

                        WindowEvent::KeyboardInput { 
                            event: KeyEvent {
                                state: ElementState::Pressed,
                                physical_key: PhysicalKey::Code(KeyCode::Escape),
                                ..
                            },
                            .. 
                        } => control_flow.exit(),

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
                }
            },

            _ => {}
        }
    });
}
