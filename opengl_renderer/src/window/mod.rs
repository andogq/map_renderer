use glutin::{
    config::ConfigTemplateBuilder,
    context::{ContextAttributesBuilder, PossiblyCurrentContext},
    display::{Display, GetGlDisplay},
    prelude::{GlConfig, GlDisplay, NotCurrentGlContextSurfaceAccessor},
    surface::{GlSurface, Surface, WindowSurface},
};
use glutin_winit::{DisplayBuilder, GlWindow};
use raw_window_handle::HasRawWindowHandle;
use winit::{
    dpi::LogicalSize,
    event::{Event, KeyboardInput, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

use self::opengl::OpenGl;

pub mod opengl;

pub enum WindowAction {
    Close,
    RequestRedraw,
}

pub struct Window {
    event_loop: EventLoop<()>,
    window: winit::window::Window,
    display: Display,
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
        let gl =
            unsafe { glow::Context::from_loader_function_cstr(|s| display.get_proc_address(s)) };

        Self {
            event_loop,
            window,
            display,
            gl_surface,
            gl_context,
            gl: OpenGl::new(gl),
        }
    }

    pub fn render(&self) -> glutin::error::Result<()> {
        self.gl.render();

        self.gl_surface.swap_buffers(&self.gl_context)
    }

    pub fn run<F>(self, mut event_handler: F) -> !
    where
        F: 'static + FnMut(KeyboardInput) -> Option<WindowAction>,
    {
        self.event_loop.run(move |event, _, control_flow| {
            control_flow.set_wait();

            match event {
                Event::WindowEvent { window_id, event } if self.window.id() == window_id => {
                    match event {
                        WindowEvent::CloseRequested => {
                            control_flow.set_exit();
                        }
                        WindowEvent::KeyboardInput { input, .. } => {
                            if let Some(action) = event_handler(input) {
                                match action {
                                    WindowAction::Close => control_flow.set_exit(),
                                    WindowAction::RequestRedraw => self.window.request_redraw(),
                                }
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        });
    }
}
