use crate::{
    ogl::{texture_buffer::TextureBufferBuilder, OpenGl, Program},
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

pub struct Renderer {
    window: Window,
    projection: Mat4,
    camera: Camera,

    render_steps: Vec<Rc<RefCell<dyn RenderStep>>>,
}

#[derive(Debug, Clone, Copy)]
pub enum Event {
    Keyboard(VirtualKeyCode),
    Click(Vec3),
}

impl Renderer {
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

    pub fn add_render_step(&mut self, render_step: Rc<RefCell<dyn RenderStep>>) {
        self.render_steps.push(render_step);
    }

    pub fn run<F>(mut self, mut event_callback: F) -> !
    where
        F: 'static + FnMut(Event),
    {
        let programs = self
            .render_steps
            .iter()
            .enumerate()
            .flat_map(|(render_step_id, render_step)| {
                let render_step = render_step.borrow();

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
                    .into_iter()
                    .enumerate()
                    .map(move |(program_offset, p)| (render_step_id, program_offset, p))
            })
            .collect::<Vec<_>>();

        let update_uniforms =
            |programs: &[(usize, usize, Rc<RefCell<Program>>)], projection: &Mat4, view: &Mat4| {
                programs
                    .iter()
                    .for_each(|(_render_step_id, _program_offset, program)| {
                        let mut program = program.borrow_mut();
                        program.set_uniform("projection", projection).unwrap();
                        program.set_uniform("view", view).unwrap();
                    });
            };

        // Provide initial uniforms
        update_uniforms(programs.as_slice(), &self.projection, &self.camera.view());

        let mut last_location = None;
        let mut mouse_location = Vec3::new(0.0, 0.0, 0.0);
        let mut dragging = false;

        self.window.run(move |event, window_info| {
            match event {
                WindowEvent::Keyboard {
                    keycode,
                    state: ElementState::Pressed,
                } => {
                    // Update camera
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

                    update_uniforms(programs.as_slice(), &self.projection, &self.camera.view());

                    // Event callback
                    event_callback(Event::Keyboard(keycode));

                    // Trigger redraw
                    return Some(WindowAction::RequestRedraw);
                }
                WindowEvent::MouseDown => {
                    dragging = true;

                    event_callback(Event::Click(mouse_location));

                    // TODO: Remove
                    programs
                        .iter()
                        .for_each(|(render_step_id, program_offset, program)| {
                            let render_step = self.render_steps[*render_step_id].borrow();
                            let vertices = render_step.get_vertices();

                            program
                                .borrow_mut()
                                .attach_vertices(vertices[*program_offset].clone(), None)
                                .unwrap();
                        });
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

                    mouse_location = plane_intersection;

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

                    update_uniforms(programs.as_slice(), &self.projection, &self.camera.view());

                    return Some(WindowAction::RequestRedraw);
                }
                _ => (),
            }

            None
        });
    }
}
