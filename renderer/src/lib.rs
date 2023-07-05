use crate::{
    ogl::{texture_buffer::TextureBufferBuilder, OpenGl, Program},
    window::{Window, WindowAction, WindowEvent},
};
use glam::{Mat4, Vec2, Vec3};
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
    /// Current position of the camera
    position: Vec3,

    /// Field of view of the camera in radians
    fov: f32,

    /// Aspect ratio of the camera
    aspect_ratio: f32,

    /// Near and far distances of the Z plane
    z_plane: (f32, f32),
}
impl Camera {
    pub fn new(fov: f32, aspect_ratio: f32, z_plane: (f32, f32)) -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 0.0),
            fov,
            aspect_ratio,
            z_plane,
        }
    }

    pub fn view(&self) -> Mat4 {
        Mat4::look_to_rh(self.position, Vec3::Y, Vec3::Z)
    }

    pub fn projection(&self) -> Mat4 {
        Mat4::perspective_rh(self.fov, self.aspect_ratio, self.z_plane.0, self.z_plane.1)
    }
}

pub struct Renderer {
    window: Window,
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
            camera: Camera::new(PI / 2.0, aspect_ratio, (0.000001, 100000.0)),
            render_steps: Vec::new(),
        }
    }

    pub fn add_render_step(&mut self, render_step: Rc<RefCell<dyn RenderStep>>) {
        self.render_steps.push(render_step);
    }

    pub fn set_camera_position(&mut self, position: Vec3) {
        self.camera.position = position;
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
        update_uniforms(
            programs.as_slice(),
            &self.camera.projection(),
            &self.camera.view(),
        );

        let mut mouse_location = Vec3::new(0.0, 0.0, 0.0);
        let mut dragging = false;
        let mut previous_normalised_screen_cursor = None;

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

                    update_uniforms(
                        programs.as_slice(),
                        &self.camera.projection(),
                        &self.camera.view(),
                    );

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
                    dragging = false;
                }
                WindowEvent::MouseMove {
                    physical_x,
                    physical_y,
                } => {
                    let size = window_info.size;
                    let x = physical_x / window_info.scale;
                    let y = physical_y / window_info.scale;

                    // https://gamedev.stackexchange.com/a/150425
                    let vertical_span = f32::tan(0.5 * self.camera.fov);

                    let normalised_screen_cursor = Vec2::new(
                        ((x / size.width as f32) * 2.0) - 1.0,
                        1.0 - ((y / size.height as f32) * 2.0),
                    );

                    // Convert camera 2D vector to view space
                    let view_cursor = Vec3::new(
                        normalised_screen_cursor.x * vertical_span,
                        1.0,
                        normalised_screen_cursor.y * vertical_span,
                    );

                    // Determine camera depth (map plane at y = 0)
                    let depth = -self.camera.position.y;

                    // Determine world cursor
                    mouse_location = self.camera.position + (view_cursor * depth);

                    if let Some(previous_normalised_screen_cursor) =
                        previous_normalised_screen_cursor
                    {
                        let screen_travel: Vec2 =
                            previous_normalised_screen_cursor - normalised_screen_cursor;

                        let world_travel = Vec3::new(
                            screen_travel.x * depth * vertical_span,
                            0.0,
                            screen_travel.y * depth * vertical_span,
                        );

                        if dragging {
                            self.camera.position += world_travel;

                            update_uniforms(
                                programs.as_slice(),
                                &self.camera.projection(),
                                &self.camera.view(),
                            );
                        }
                    }

                    previous_normalised_screen_cursor = Some(normalised_screen_cursor);

                    return Some(WindowAction::RequestRedraw);
                }
                WindowEvent::Scroll { x: _, y } => {
                    if y.abs() != 0.0 {
                        // Scroll values are kind of arbitrary, but seem to increase with more
                        // 'momentum' or speed on the mouse wheel/trackpad. `zoom_magnitude_max`
                        // provides an upperbound for this value, allowing it to be reduced to a
                        // value between 0 and 1, causing the scroll speed to increase or
                        // decrease depending on how fast the user is scrolling.
                        let zoom_magnitude_max = 15.0;
                        let zoom_speed_threshold =
                            y.abs().min(zoom_magnitude_max) / zoom_magnitude_max;

                        // Scale will be how much the zoom level will change as a result of the
                        // zoom event. It must atleast be the same (1.0), and cannot change by more
                        // than 25% of it's current value (0.25), the percentage of which is
                        // determined by the scroll speed, as discussed above.
                        let scale = 1.0 + (-0.25 * zoom_speed_threshold * y.signum());
                        self.camera.position.y = (self.camera.position.y * scale).min(-1.0);

                        update_uniforms(
                            programs.as_slice(),
                            &self.camera.projection(),
                            &self.camera.view(),
                        );

                        return Some(WindowAction::RequestRedraw);
                    }
                }
                _ => (),
            }

            None
        });
    }
}

