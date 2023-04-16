use crate::{
    ogl::{
        texture_buffer::TextureBufferBuilder, DrawType, OpenGl, Program, VertexFormat, VertexType,
    },
    window::{Window, WindowAction, WindowEvent},
};
use glam::{Mat4, Vec3, Vec4};
use opengl::ImageFormat;
use std::{cell::RefCell, f32::consts::PI, rc::Rc};
use winit::event::{ElementState, VirtualKeyCode};

pub mod ogl;
pub mod render_steps;
pub mod window;

pub trait RenderStep {
    fn build_programs(&self, gl: &mut OpenGl) -> Vec<Rc<RefCell<Program>>>;
    fn get_vertices(&self) -> Vec<Vec<u8>>;
    fn get_texture_buffer(&self) -> Option<Vec<u8>> {
        None
    }
}

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

pub struct Renderer<'a> {
    window: Window,
    projection: Mat4,
    camera: Camera,

    render_steps: Vec<Box<dyn RenderStep + 'a>>,
}

impl<'a> Renderer<'a> {
    pub fn with_window(window: window::Window) -> Self {
        let aspect_ratio = {
            let size = window.get_size();
            size.1 as f32 / size.0 as f32
        };

        Self {
            window,
            projection: Mat4::perspective_rh(PI / 2.0, aspect_ratio, 0.001, 500.0),
            camera: Camera::new(),
            render_steps: Vec::new(),
        }
    }

    pub fn add_render_step<R>(&mut self, render_step: R)
    where
        R: RenderStep + 'a,
    {
        self.render_steps.push(Box::new(render_step));
    }

    pub fn run(mut self) -> ! {
        let programs = self
            .render_steps
            .iter()
            .flat_map(|render_step| {
                let programs = render_step.build_programs(&mut self.window.gl);
                let vertices = render_step.get_vertices();

                let texture_buffer = render_step.get_texture_buffer().map(|data| {
                    let texture_buffer = self
                        .window
                        .gl
                        .create_texture(TextureBufferBuilder::new().with_format(ImageFormat::R32F))
                        .unwrap();

                    texture_buffer.set_data(&data);

                    Rc::new(texture_buffer)
                });

                for (program, vertices) in programs.iter().zip(vertices) {
                    let mut program = program.borrow_mut();

                    // Attach vertices
                    program.attach_vertices(vertices, None).unwrap();

                    if let Some(texture_buffer) = texture_buffer.as_ref() {
                        // Attach texture buffer
                        program.attach_texture_buffer(texture_buffer.clone());
                    }
                }

                programs
            })
            .collect::<Vec<_>>();

        let update_uniforms =
            |programs: &[Rc<RefCell<Program>>], projection: &Mat4, view: &Mat4| {
                programs.iter().for_each(|program| {
                    let mut program = program.borrow_mut();
                    program.set_uniform("projection", projection).unwrap();
                    program.set_uniform("view", view).unwrap();
                });
            };

        // Provide initial uniforms
        update_uniforms(programs.as_slice(), &self.projection, &self.camera.view());

        let line_program = self
            .window
            .gl
            .add_program(
                Program::from_directory("line")
                    .unwrap()
                    .with_format(&[
                        VertexFormat::new(1, VertexType::UInt),
                        VertexFormat::new(3, VertexType::Float),
                        VertexFormat::new(1, VertexType::Float),
                        VertexFormat::new(3, VertexType::Float),
                        VertexFormat::new(1, VertexType::Float),
                    ])
                    .with_draw_type(DrawType::LineStrip),
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

                    update_uniforms(programs.as_slice(), &self.projection, &self.camera.view());

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

                            update_uniforms(
                                programs.as_slice(),
                                &self.projection,
                                &self.camera.view(),
                            );
                        } else {
                            last_location = Some(plane_intersection);
                        }
                    } else {
                        last_location = None;
                    }

                    return Some(WindowAction::RequestRedraw);
                }
                WindowEvent::Scroll { x: _, y } => {
                    self.camera.position.y += y / 10.0;

                    line_program
                        .set_uniform("view", &self.camera.view())
                        .unwrap();

                    update_uniforms(programs.as_slice(), &self.projection, &self.camera.view());

                    return Some(WindowAction::RequestRedraw);
                }
                _ => (),
            }

            None
        });
    }
}
