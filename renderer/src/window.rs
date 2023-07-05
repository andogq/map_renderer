use std::time::{Duration, Instant};

use crate::ogl::OpenGl;
use glutin::{
    config::ConfigTemplateBuilder,
    context::{ContextAttributesBuilder, PossiblyCurrentContext},
    display::GetGlDisplay,
    prelude::{GlConfig, GlDisplay, NotCurrentGlContextSurfaceAccessor},
    surface::{GlSurface, Surface, WindowSurface},
};
use glutin_winit::{DisplayBuilder, GlWindow};
use opengl::Context;
use raw_window_handle::HasRawWindowHandle;
use winit::{
    dpi::LogicalSize,
    event::{
        ElementState, Event, KeyboardInput, MouseButton, MouseScrollDelta, TouchPhase,
        VirtualKeyCode,
    },
    event_loop::EventLoop,
    window::WindowBuilder,
};

const FRAME_TARGET: usize = 120;

pub enum WindowAction {
    Close,
    RequestRedraw,
}

pub enum WindowEvent {
    Keyboard {
        keycode: VirtualKeyCode,
        state: ElementState,
    },
    MouseMove {
        physical_x: f32,
        physical_y: f32,
    },
    MouseDown,
    MouseUp,
    Scroll {
        x: f32,
        y: f32,
    },
}

pub struct WindowSize {
    pub height: u32,
    pub width: u32,
}
impl From<(usize, usize)> for WindowSize {
    fn from(size: (usize, usize)) -> Self {
        Self {
            height: size.0 as u32,
            width: size.1 as u32,
        }
    }
}
impl<P: Into<u32>> From<LogicalSize<P>> for WindowSize {
    fn from(value: LogicalSize<P>) -> Self {
        Self {
            width: value.width.into(),
            height: value.height.into(),
        }
    }
}

pub struct WindowInfo {
    pub size: WindowSize,
    pub scale: f32,
}

impl TryFrom<winit::event::WindowEvent<'_>> for WindowEvent {
    type Error = ();

    fn try_from(event: winit::event::WindowEvent<'_>) -> Result<Self, Self::Error> {
        match event {
            winit::event::WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                ..
            } => Ok(Self::Keyboard { keycode, state }),
            winit::event::WindowEvent::CursorMoved { position, .. } => Ok(Self::MouseMove {
                physical_x: position.x as f32,
                physical_y: position.y as f32,
            }),
            winit::event::WindowEvent::MouseInput { state, button, .. } => match (button, state) {
                (MouseButton::Left, ElementState::Pressed) => Ok(Self::MouseDown),
                (MouseButton::Left, ElementState::Released) => Ok(Self::MouseUp),
                _ => Err(()),
            },
            winit::event::WindowEvent::MouseWheel {
                delta: MouseScrollDelta::PixelDelta(position),
                phase: TouchPhase::Moved,
                ..
            } => Ok(Self::Scroll {
                x: position.x as f32,
                y: position.y as f32,
            }),
            _ => Err(()),
        }
    }
}

pub struct Window {
    event_loop: Option<EventLoop<()>>,
    window: winit::window::Window,
    gl_surface: Surface<WindowSurface>,
    gl_context: PossiblyCurrentContext,
    pub gl: OpenGl,
}

impl Window {
    pub fn new(size: (usize, usize)) -> Self {
        let event_loop = EventLoop::new();

        let (mut window, gl_config) = DisplayBuilder::new()
            .with_window_builder(Some({
                // This window builder will be used to create a new window
                let size = LogicalSize::new(size.1 as f32, size.0 as f32);

                WindowBuilder::new()
                    .with_title("OpenGL Renderer")
                    .with_inner_size(size)
                    .with_min_inner_size(size)
            }))
            .build(&event_loop, ConfigTemplateBuilder::new(), |configs| {
                // Select the best config out of the available ones
                configs
                    .reduce(|ideal, config| {
                        // Select the config with the greatest sampling
                        if config.num_samples() > ideal.num_samples() {
                            config
                        } else {
                            ideal
                        }
                    })
                    .unwrap()
            })
            .unwrap();

        // Save the window and display for future reference
        let window = window.take().unwrap();
        let display = gl_config.display();

        // Create the OpenGL surface on the window
        let gl_surface = unsafe {
            display
                .create_window_surface(&gl_config, &window.build_surface_attributes(<_>::default()))
                .unwrap()
        };

        // Create the OpenGL context within the surface
        let gl_context = match unsafe {
            display.create_context(
                &gl_config,
                &ContextAttributesBuilder::new().build(Some(window.raw_window_handle())),
            )
        } {
            Ok(gl_context) => {
                // Context successfully created, make sure that it's the current context
                Some(gl_context.make_current(&gl_surface).unwrap())
            }
            Err(e) => {
                eprintln!("Problem creating gl context");
                eprintln!("{:#?}", e);

                None
            }
        }
        .expect("gl_context present");

        // Get the OpenGL bindings from the display
        let context = Context::load(|s| display.get_proc_address(s));

        Self {
            event_loop: Some(event_loop),
            window,
            gl_surface,
            gl_context,
            gl: OpenGl::new(context),
        }
    }

    pub fn get_size(&self) -> (u32, u32) {
        let size = self
            .window
            .inner_size()
            .to_logical::<u32>(self.window.scale_factor());

        (size.height, size.width)
    }

    pub fn render(&self) -> glutin::error::Result<()> {
        self.gl.render();

        self.gl_surface.swap_buffers(&self.gl_context)
    }

    pub fn run<F>(mut self, mut event_handler: F) -> !
    where
        F: 'static + FnMut(WindowEvent, WindowInfo) -> Option<WindowAction>,
    {
        let event_loop = self.event_loop.take().unwrap();

        let min_frame_time = Duration::from_secs(1) / FRAME_TARGET as u32;
        let mut last_frame_time = None;
        let mut redraw_pending = false;

        event_loop.run(move |event, _, control_flow| {
            control_flow.set_wait();

            // Handle events
            let forward_event = match event {
                Event::RedrawRequested(window_id) if self.window.id() == window_id => {
                    // Attempt to re-render
                    let now = Instant::now();
                    let frame_time = last_frame_time.map(|last| now - last);
                    if frame_time
                        .map(|frame_time| frame_time > min_frame_time)
                        .unwrap_or(true)
                    {
                        // Record current frame time
                        last_frame_time = Some(now);

                        // println!(
                        //     "fps: {}",
                        //     1.0 / frame_time.unwrap_or_default().as_secs_f32()
                        // );

                        // TODO: bad
                        self.render().unwrap();

                        redraw_pending = false;
                    } else if let Some(frame_time) = frame_time {
                        redraw_pending = true;
                        control_flow.set_wait_timeout(min_frame_time - frame_time)
                    }

                    None
                }
                Event::WindowEvent { window_id, event } if self.window.id() == window_id => {
                    match event {
                        winit::event::WindowEvent::CloseRequested => {
                            control_flow.set_exit();
                            None
                        }
                        window_event => window_event.try_into().ok(),
                    }
                }
                // Event::DeviceEvent { event, .. } => event.try_into().ok(),
                _ => None,
            };

            if let Some(event) = forward_event {
                let window_scale = self.window.scale_factor();
                let window_info = WindowInfo {
                    size: self
                        .window
                        .inner_size()
                        .to_logical::<u32>(window_scale)
                        .into(),
                    scale: window_scale as f32,
                };

                if let Some(action) = event_handler(event, window_info) {
                    match action {
                        WindowAction::Close => control_flow.set_exit(),
                        WindowAction::RequestRedraw => {
                            self.window.request_redraw();
                        }
                    }
                }
            }

            if redraw_pending {
                // Will end up in here if wait event runs through
                self.window.request_redraw();
            }
        });
    }
}
