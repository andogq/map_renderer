use std::f32::consts::PI;

use glam::{Mat4, Vec3};
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode};

use crate::{
    opengl::{OpenGlError, ShaderType, VertexFormat, VertexType},
    window::{Window, WindowAction},
};

struct Camera {
    position: Vec3,
}
impl Camera {
    pub fn new() -> Self {
        Self {
            position: Vec3::new(0.0, 10.0, 0.0),
        }
    }

    pub fn view(&self) -> Mat4 {
        Mat4::look_to_rh(self.position, Vec3::NEG_Y, Vec3::Z)
    }
}

pub struct World {
    window: Window,
    projection: Mat4,
    camera: Camera,
}

impl World {
    pub fn with_window(window: Window) -> Self {
        let aspect_ratio = {
            let size = window.get_size();
            size.1 as f32 / size.0 as f32
        };

        Self {
            window,
            projection: Mat4::perspective_rh(PI / 2.0, aspect_ratio, 1.0, 50.0),
            camera: Camera::new(),
        }
    }

    pub fn run(mut self) -> ! {
        let program = self.window.gl.create_program();

        {
            let mut program = program.borrow_mut();

            program
                .attach_shader(ShaderType::Vertex, VERTEX_SHADER)
                .unwrap();
            program
                .attach_shader(ShaderType::Fragment, FRAGMENT_SHADER)
                .unwrap();
            program.link().unwrap();

            program
                .attach_vertices(
                    &[
                        0.5, 0.0, 1.0, // V1
                        0.0, 0.0, 0.0, // V2
                        1.0, 0.0, 0.0, // V3
                    ],
                    &[VertexFormat::new(3, VertexType::Float)],
                )
                .unwrap();

            program.set_uniform("projection", &self.projection).unwrap();
            program.set_uniform("view", &self.camera.view()).unwrap();
        }

        self.window.run(move |event| {
            let mut program = program.borrow_mut();

            if let KeyboardInput {
                virtual_keycode: Some(keycode),
                state: ElementState::Pressed,
                ..
            } = event
            {
                match keycode {
                    VirtualKeyCode::Escape => {
                        return Some(WindowAction::Close);
                    }
                    VirtualKeyCode::W => {
                        self.camera.position.z += 1.0;
                    }
                    VirtualKeyCode::S => {
                        self.camera.position.z -= 1.0;
                    }
                    VirtualKeyCode::A => {
                        self.camera.position.x += 1.0;
                    }
                    VirtualKeyCode::D => {
                        self.camera.position.x -= 1.0;
                    }
                    _ => (),
                }

                // Update uniforms
                program.set_uniform("view", &self.camera.view()).unwrap();

                // Trigger redraw
                return Some(WindowAction::RequestRedraw);
            }

            None
        });
    }
}

const VERTEX_SHADER: &str = r#"
#version 410

layout(location = 0) in vec3 position;

uniform mat4 projection;
uniform mat4 view;

void main() {
    gl_Position = projection * view * vec4(position, 1.0);
    // gl_Position = vec4(position.x - 0.5, position.y - 0.5, 0.0, 1.0);
}
"#;

const FRAGMENT_SHADER: &str = r#"
#version 410

out vec4 color;

void main() {
    color = vec4(0.0, 0.72, 0.40, 1.0);
    // color = vec4(0.0, 0.0, 0.0, 1.0);
}
"#;
