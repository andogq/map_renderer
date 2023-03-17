use window::opengl::OpenGlError;
use winit::event::VirtualKeyCode;

use crate::window::{
    opengl::{ShaderType, VertexFormat, VertexType},
    Window, WindowAction,
};

mod window;

const WINDOW_SIZE: (usize, usize) = (480, 720);

fn main() -> Result<(), OpenGlError> {
    let mut window = Window::new(WINDOW_SIZE);

    println!("{:?}", window.gl.get_info());

    let program = window.gl.create_program();

    {
        let mut program = program.borrow_mut();

        program.attach_shader(ShaderType::Vertex, VERTEX_SHADER)?;
        program.attach_shader(ShaderType::Fragment, FRAGMENT_SHADER)?;

        program.link()?;

        program.attach_vertices(
            &[
                0.5f32, 1.0f32, 0.0, // V1
                0.0f32, 0.0f32, 0.5, // V2
                1.0f32, 0.0f32, 1.0, // V3
            ],
            &[
                VertexFormat::new(2, VertexType::Float),
                VertexFormat::new(1, VertexType::Float),
            ],
        )?;

        program.set_uniform("blue", 0.5)?;
    }

    window.render().unwrap();

    window.run(|event| {
        if let Some(keycode) = event.virtual_keycode {
            match keycode {
                VirtualKeyCode::Escape => {
                    return Some(WindowAction::Close);
                }
                _ => (),
            }
        }

        None
    });

    // let program = {
    //     // Create the program
    //     let program = unsafe { gl.create_program().expect("program to be created") };
    //
    //     let shaders = [
    //         (glow::VERTEX_SHADER, VERTEX_SHADER),
    //         (glow::FRAGMENT_SHADER, FRAGMENT_SHADER),
    //     ]
    //     .into_iter()
    //     .map(|(shader_type, source)| unsafe {
    //         let shader = gl.create_shader(shader_type).expect("shader to be created");
    //         gl.shader_source(shader, source);
    //         gl.compile_shader(shader);
    //
    //         if !gl.get_shader_compile_status(shader) {
    //             panic!("{}", gl.get_shader_info_log(shader));
    //         }
    //
    //         gl.attach_shader(program, shader);
    //
    //         shader
    //     })
    //     .collect::<Vec<_>>();
    //
    //     unsafe {
    //         gl.link_program(program);
    //         if !gl.get_program_link_status(program) {
    //             panic!("{}", gl.get_program_info_log(program));
    //         }
    //     }
    //
    //     // Cleanup
    //     for shader in shaders {
    //         unsafe {
    //             gl.detach_shader(program, shader);
    //             gl.delete_shader(shader);
    //         }
    //     }
    //
    //     program
    // };
    //
    // unsafe {
    //     gl.use_program(Some(program));
    // }
    //
    // let (vertex_buffer, vertex_array) = unsafe {
    //     let triangle_vertices = [
    //         0.5f32, 1.0f32, 0.0, // V1
    //         0.0f32, 0.0f32, 0.5, // V2
    //         1.0f32, 0.0f32, 1.0, // V3
    //     ];
    //     let triangle_vertices_u8: &[u8] = core::slice::from_raw_parts(
    //         triangle_vertices.as_ptr() as *const u8,
    //         triangle_vertices.len() * core::mem::size_of::<f32>(),
    //     );
    //
    //     let vertex_buffer = gl.create_buffer().unwrap();
    //     gl.bind_buffer(glow::ARRAY_BUFFER, Some(vertex_buffer));
    //     gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, triangle_vertices_u8, glow::STATIC_DRAW);
    //
    //     let vertex_array = gl.create_vertex_array().unwrap();
    //     gl.bind_vertex_array(Some(vertex_array));
    //     gl.enable_vertex_attrib_array(0);
    //     gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 12, 0);
    //
    //     gl.enable_vertex_attrib_array(1);
    //     gl.vertex_attrib_pointer_f32(1, 1, glow::FLOAT, false, 12, 8);
    //
    //     (vertex_buffer, vertex_array)
    // };
    //
    // let blue_uniform_location = unsafe { gl.get_uniform_location(program, "blue") };
    //
    // let mut wireframe = false;
    // let mut blue: f32 = 0.0;
    //
    // event_loop.run(move |event, _, control_flow| {
    //     control_flow.set_wait();
    //
    //     #[allow(clippy::collapsible_match)]
    //     match event {
    //         Event::RedrawRequested(window_id) if window_id == window.id() => {
    //             // Redraw window
    //             println!("Redraw requested");
    //
    //             unsafe {
    //                 gl.clear_color(1.0, 1.0, 1.0, 1.0);
    //                 gl.clear(glow::COLOR_BUFFER_BIT);
    //
    //                 blue = blue.min(1.0).max(0.0);
    //                 gl.uniform_1_f32(blue_uniform_location.as_ref(), blue);
    //
    //                 gl.polygon_mode(
    //                     glow::FRONT_AND_BACK,
    //                     if wireframe { glow::LINE } else { glow::FILL },
    //                 );
    //
    //                 gl.draw_arrays(glow::TRIANGLES, 0, 3);
    //             };
    //
    //             gl_surface.swap_buffers(&gl_context).unwrap();
    //         }
    //         Event::WindowEvent { window_id, event } if window_id == window.id() => match event {
    //             WindowEvent::KeyboardInput {
    //                 input:
    //                     KeyboardInput {
    //                         virtual_keycode: Some(keycode),
    //                         state: ElementState::Pressed,
    //                         ..
    //                     },
    //                 ..
    //             } =>
    //             {
    //                 #[allow(clippy::single_match)]
    //                 match keycode {
    //                     VirtualKeyCode::Escape => control_flow.set_exit(),
    //                     VirtualKeyCode::Space => {
    //                         wireframe = !wireframe;
    //                         window.request_redraw();
    //                     }
    //                     VirtualKeyCode::Equals => {
    //                         blue += 0.1;
    //                         window.request_redraw();
    //                     }
    //                     VirtualKeyCode::Minus => {
    //                         blue -= 0.1;
    //                         window.request_redraw();
    //                     }
    //                     _ => {}
    //                 }
    //             }
    //             WindowEvent::CloseRequested => control_flow.set_exit(),
    //             _ => {}
    //         },
    //         _ => {}
    //     }
    // })
}

const VERTEX_SHADER: &str = r#"
#version 410

layout(location = 0) in vec2 in_position;
layout(location = 1) in float in_green;

layout(location = 0) out vec2 position;
layout(location = 1) out float green;

void main() {
    position = in_position;
    green = in_green;

    gl_Position = vec4(in_position - 0.5, 0.0, 1.0);
}
"#;

const FRAGMENT_SHADER: &str = r#"
#version 410

layout(location = 0) in vec2 position;
layout(location = 1) in float green;

out vec4 color;

uniform float blue;

void main() {
    color = vec4(0.0, green, blue, 1.0);
}
"#;
