use glutin::{
    config::ConfigTemplateBuilder,
    context::ContextAttributesBuilder,
    display::GetGlDisplay,
    prelude::{GlDisplay, NotCurrentGlContextSurfaceAccessor},
};
use glutin_winit::{DisplayBuilder, GlWindow};
use raw_window_handle::HasRawWindowHandle;
use winit::{event_loop::EventLoop, window::WindowBuilder};

use opengl::*;

fn main() {
    // Initialisation
    let context = load_gl_bindings();

    // Tests
    get_string(context);
}

fn load_gl_bindings() -> Context {
    let event_loop = EventLoop::new();

    let (mut window, gl_config) = DisplayBuilder::new()
        .with_window_builder(Some(WindowBuilder::new()))
        .build(&event_loop, ConfigTemplateBuilder::new(), |mut configs| {
            configs.next().unwrap()
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
    let _gl_context = match unsafe {
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

    // Load OpenGL bindings
    Context::load(|s| display.get_proc_address(s))
}

fn get_string(context: Context) {
    [
        StringName::Vendor,
        StringName::Renderer,
        StringName::Version,
        StringName::ShaderLanguageVersion,
    ]
    .into_iter()
    .for_each(|string_name| {
        let value = context.get_string(string_name).unwrap();
        println!("{string_name:?}: {value}");
    });
}
