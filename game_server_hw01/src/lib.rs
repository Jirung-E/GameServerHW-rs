use winit::{
    event::*,
    event_loop::{EventLoop, EventLoopWindowTarget},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowBuilder},
};


struct State<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    // The window must be declared after the surface so
    // it gets dropped after it as the surface contains
    // unsafe references to the window's resources.
    window: &'a Window,
    background_color: wgpu::Color,
}


impl<'a> State<'a> {
    // Creating some of the wgpu types requires async code
    async fn new(window: &'a Window) -> State<'a> {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            #[cfg(not(target_arch="wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch="wasm32")]
            backends: wgpu::Backends::GL,
            ..Default::default()
        });
        
        let surface = instance.create_surface(window).unwrap();

        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),     // PowerPreference::HighPerformance
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.unwrap();

        // // instead of request_adapter
        // let adapter = instance
        //     .enumerate_adapters(wgpu::Backends::all())   // isn't available on WASM
        //     .into_iter()
        //     .filter(|adapter| {
        //         // Check if this adapter supports our surface
        //         adapter.is_surface_supported(&surface)
        //     })
        //     .next()
        //     .unwrap()

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                // WebGL doesn't support all of wgpu's features, so if
                // we're building for the web, we'll have to disable some.
                required_limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                label: None,
            },
            None, // Trace path
        ).await.unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result in all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };


        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            background_color: wgpu::Color {
                r: 0.1,
                g: 0.2,
                b: 0.3,
                a: 1.0,
            },
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn input(&mut self, _event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self) {
        
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.background_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
        }
    
        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    
        Ok(())
    }
}








///////////////////////////////////////////////////////////////////////////////











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