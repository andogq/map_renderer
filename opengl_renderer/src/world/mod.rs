use std::f32::consts::PI;

use glam::{Mat4, Vec3, Vec4};
use winit::event::{ElementState, VirtualKeyCode};

use crate::{
    opengl::{DrawType, Program, ShaderType, VertexFormat, VertexType},
    window::{Window, WindowAction, WindowEvent},
};

struct Camera {
    position: Vec3,
}
impl Camera {
    pub fn new() -> Self {
        Self {
            position: Vec3::new(0.0, 20.0, 5.0),
        }
    }

    pub fn view(&self) -> Mat4 {
        Mat4::look_to_rh(self.position, -self.position, Vec3::Y)
    }
}

pub struct World {
    window: Window,
    projection: Mat4,
    camera: Camera,
}

fn generate_grid(size: u32, spacing: f32) -> Vec<f32> {
    let mut grid = Vec::new();

    let start = -0.5 * size as f32 * spacing;

    for i in 0..size {
        for j in 0..size {
            let x = start + (i as f32 * spacing);
            let z = start + (j as f32 * spacing);

            grid.extend_from_slice(&[x, 0.0, z]);
        }
    }

    grid
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
        let program = self
            .window
            .gl
            .add_program(
                Program::builder()
                    .with_shader(ShaderType::Vertex, VERTEX_SHADER)
                    .with_shader(ShaderType::Fragment, FRAGMENT_SHADER)
                    .with_format(&[VertexFormat::new(3, VertexType::Float)])
                    .with_draw_type(DrawType::Triangles),
            )
            .unwrap();
        let point_program = self
            .window
            .gl
            .add_program(
                Program::builder()
                    .with_shader(ShaderType::Vertex, VERTEX_SHADER)
                    .with_shader(ShaderType::Fragment, FRAGMENT_SHADER)
                    .with_format(&[VertexFormat::new(3, VertexType::Float)])
                    .with_draw_type(DrawType::Points),
            )
            .unwrap();

        {
            let mut program = program.borrow_mut();

            program
                .attach_vertices(&[
                    1.0, 1.0, 2.0, // V1
                    0.0, 1.0, 0.0, // V2
                    2.0, 1.0, 0.0, // V3
                ])
                .unwrap();

            program.set_uniform("projection", &self.projection).unwrap();
            program.set_uniform("view", &self.camera.view()).unwrap();
        }

        {
            let mut program = point_program.borrow_mut();

            program.set_uniform("projection", &self.projection).unwrap();
            program.set_uniform("view", &self.camera.view()).unwrap();
        }

        self.window.run(move |event, window_info| {
            let mut program = program.borrow_mut();
            let mut point_program = point_program.borrow_mut();

            match event {
                WindowEvent::Keyboard {
                    keycode,
                    state: ElementState::Pressed,
                } => {
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
                    point_program
                        .set_uniform("view", &self.camera.view())
                        .unwrap();

                    // Trigger redraw
                    return Some(WindowAction::RequestRedraw);
                }
                WindowEvent::MouseMove {
                    physical_x,
                    physical_y,
                } => {
                    let size = window_info.size;
                    let x = physical_x / window_info.scale;
                    let y = physical_y / window_info.scale;

                    // Step 1
                    let x = ((x / size.width as f32) * 2.0) - 1.0;
                    let y = 1.0 - ((y / size.height as f32) * 2.0);

                    // Step 2
                    let mouse_ndc = Vec4::new(x, y, 1.0, 1.0);

                    // Step 3
                    let inverse_mvp = self.projection.mul_mat4(&self.camera.view()).inverse();

                    // Step 4
                    let mouse_world = inverse_mvp * mouse_ndc;

                    // Step 5
                    let mouse_position = Vec3::new(
                        mouse_world.x / mouse_world.w,
                        mouse_world.y / mouse_world.w,
                        mouse_world.z / mouse_world.w,
                    );

                    // Ray pointing from camera to the mouse_position (could be at any depth)
                    let normalised_ray = (mouse_position - self.camera.position).normalize();

                    // Extend normalised_ray from camera to intersect with plane (y = 0)
                    let ray_point = self.camera.position;
                    let plane_normal = Vec3::Y;
                    let plane_point = Vec3::ZERO;
                    let plane_intersection = ray_point
                        + ((plane_normal.dot(plane_point - ray_point))
                            / (plane_normal.dot(normalised_ray))
                            * normalised_ray);

                    point_program
                        .attach_vertices(
                            &[
                                generate_grid(16, 1.0),
                                plane_intersection.to_array().to_vec(),
                            ]
                            .concat(),
                        )
                        .unwrap();

                    return Some(WindowAction::RequestRedraw);
                }
                _ => (),
            }

            None
        });
    }
}

const VERTEX_SHADER: &str = r#"
#version 410

layout(location = 0) in vec3 position;

out vec3 out_position;

uniform mat4 projection;
uniform mat4 view;

void main() {
    out_position = position;
    gl_Position = projection * view * vec4(position, 1.0);
    gl_PointSize = 5.0;
}
"#;

const FRAGMENT_SHADER: &str = r#"
#version 410

in vec3 out_position;
out vec4 color;

void main() {
    if (out_position.x == 0) {
        color = vec4(0.0, 0.0, 1.0, 1.0);
    } else if (out_position.z == 0) {
        color = vec4(1.0, 0.0, 0.0, 1.0);
    } else {
        color = vec4(0.6, 0.6, 0.6, 1.0);
    }
}
"#;
