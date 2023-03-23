use self::line::Line;
use crate::{
    opengl::{DrawType, Program, VertexFormat, VertexType},
    window::{Window, WindowAction, WindowEvent},
};
use glam::{Mat4, Vec3, Vec4};
use std::f32::consts::PI;
use winit::event::{ElementState, VirtualKeyCode};

pub mod line;

struct Camera {
    position: Vec3,
}
impl Camera {
    pub fn new() -> Self {
        Self {
            position: Vec3::new(0.0, -20.0, 0.0),
        }
    }

    pub fn view(&self) -> Mat4 {
        Mat4::look_to_rh(self.position, Vec3::Y, Vec3::Z)
    }
}

pub struct World {
    window: Window,
    projection: Mat4,
    camera: Camera,

    lines: Vec<Line>,
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
            lines: Vec::new(),
        }
    }

    pub fn add_line(&mut self, line: Line) {
        self.lines.push(line);
    }

    pub fn run(mut self) -> ! {
        let line_program = self
            .window
            .gl
            .add_program(
                Program::from_directory("line")
                    .unwrap()
                    .with_format(&[
                        VertexFormat::new(3, VertexType::Float),
                        VertexFormat::new(1, VertexType::Float),
                        VertexFormat::new(3, VertexType::Float),
                    ])
                    .with_draw_type(DrawType::Lines),
            )
            .unwrap();

        let mut last_location = None;
        let mut dragging = false;

        {
            let mut line_program = line_program.borrow_mut();

            line_program
                .set_uniform("projection", &self.projection)
                .unwrap();
            line_program
                .set_uniform("view", &self.camera.view())
                .unwrap();

            let line_vertices = self
                .lines
                .iter()
                .map(|line| line.flatten())
                .collect::<Vec<_>>()
                .concat();
            line_program.attach_vertices(&line_vertices).unwrap();
        }

        self.window.run(move |event, window_info| {
            let mut line_program = line_program.borrow_mut();

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

                    line_program
                        .set_uniform("view", &self.camera.view())
                        .unwrap();

                    // Trigger redraw
                    return Some(WindowAction::RequestRedraw);
                }
                WindowEvent::MouseDown => {
                    dragging = true;
                }
                WindowEvent::MouseUp => {
                    last_location = None;
                    dragging = false;
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

                    dbg!(plane_intersection);

                    if dragging {
                        if let Some(last) = last_location {
                            let translation = last - plane_intersection;
                            let target_transition = self.camera.position + translation;

                            let distance = self.camera.position.distance(target_transition);
                            let t = f32::clamp(0.2 * distance, 0.01, 1.0);

                            self.camera.position =
                                glam::Vec3::lerp(self.camera.position, target_transition, t);
                            last_location = Some(
                                plane_intersection - (self.camera.position - target_transition),
                            );

                            line_program
                                .set_uniform("view", &self.camera.view())
                                .unwrap();
                        } else {
                            last_location = Some(plane_intersection);
                        }
                    } else {
                        last_location = None;
                    }

                    return Some(WindowAction::RequestRedraw);
                }
                _ => (),
            }

            None
        });
    }
}
