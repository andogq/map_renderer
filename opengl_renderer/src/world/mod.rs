use self::{line::Line, polygon::Polygon};
use crate::{
    opengl::{DrawArrays, DrawType, OpenGl, Program, VertexData, VertexFormat, VertexType},
    window::{Window, WindowAction, WindowEvent},
};
use glam::{Mat4, Vec3, Vec4};
use std::{cell::RefCell, f32::consts::PI, rc::Rc};
use winit::event::{ElementState, VirtualKeyCode};

pub mod line;
pub mod path;
pub mod polygon;

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

#[derive(Clone, Debug)]
pub struct Stroke {
    width: f32,
    dash: Option<f32>,
    color: Vec3,
}
impl Stroke {
    pub fn new(width: f32, color: Vec3) -> Self {
        Self {
            width,
            color,
            dash: None,
        }
    }

    pub fn with_dash(mut self, dash: f32) -> Self {
        self.dash = Some(dash);

        self
    }
}

pub fn point_in_triangle(p: Vec3, a: Vec3, b: Vec3, c: Vec3) -> bool {
    // https://www.youtube.com/watch?v=HYAgJN3x4GA
    let w1 = ((a.x * (c.z - a.z)) + ((p.z - a.z) * (c.x - a.x)) - (p.x * (c.z - a.z)))
        / (((b.z - a.z) * (b.x - a.x)) - ((b.x - a.x) * (c.z - a.z)));
    let w2 = (p.z - a.z - (w1 * (b.z - a.z))) / (c.z - a.z);

    w1 >= 0.0 && w2 >= 0.0 && (w1 + w2) <= 1.0
}

#[derive(Clone, Debug)]
pub struct Fill {
    indexes: Vec<usize>,
    fill: Vec3,
}
impl Fill {
    pub fn new(fill: Vec3, outline: &[Vec3]) -> Self {
        let mut indexes = Vec::new();
        let mut remaining_indexes = (0..outline.len()).collect::<Vec<_>>();

        let mut i = 0;
        'point_loop: while remaining_indexes.len() > 3 {
            let left_i = i % remaining_indexes.len();
            let center_i = (i + 1) % remaining_indexes.len();
            let right_i = (i + 2) % remaining_indexes.len();

            let left = outline[left_i];
            let center = outline[center_i];
            let right = outline[right_i];

            // Check angle between center point
            let left_side = left - center;
            let right_side = right - center;

            // Assumes that polygon is on y=0 plane
            let cross = left_side.cross(right_side);

            if cross.y < 0.0 {
                // Internal angle
                for &index in remaining_indexes.iter() {
                    let p = outline[index];
                    if p == left || p == center || p == right {
                        continue;
                    }

                    if point_in_triangle(p, center, left, right) {
                        i += 1;
                        continue 'point_loop;
                    }
                }

                // If reached here, everything is valid
                indexes.extend_from_slice({
                    let l = remaining_indexes[left_i];
                    let r = remaining_indexes[right_i];
                    &[l, remaining_indexes.remove(center_i), r]
                });
            }

            i += 1;
        }

        indexes.extend(remaining_indexes.into_iter());

        Fill { indexes, fill }
    }
}

pub trait CanvasObject {
    fn get_vertices(&self) -> Vec<Vec3>;

    fn get_stroke(&self) -> Option<Stroke>;
    fn get_fill(&self) -> Option<Fill>;
}

pub struct CanvasProgram<'a> {
    objects: Vec<Box<dyn CanvasObject + 'a>>,
}

impl<'a> CanvasProgram<'a> {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }

    pub fn add_object<O>(&mut self, object: O)
    where
        O: CanvasObject + 'a,
    {
        self.objects.push(Box::new(object));
    }
}

impl RenderStep for CanvasProgram<'_> {
    fn get_vertices(&self) -> Vec<Vec<u8>> {
        let (fill, outline): (Vec<Vec<u8>>, Vec<Vec<u8>>) = self
            .objects
            .iter()
            .enumerate()
            .map(|(id, object)| {
                let id = id as u32;

                let outline_vertices = {
                    let mut v = object.get_vertices();

                    // Join stroke back to start
                    v.push(*v.first().unwrap());
                    v
                };

                let fill_vertices = object
                    .get_fill()
                    .as_ref()
                    .map(|fill| {
                        fill.indexes
                            .iter()
                            .map(|&i| outline_vertices[i])
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();

                // Get the vertices for the object and give them an ID
                (
                    fill_vertices
                        .into_iter()
                        .flat_map(move |vertex| {
                            [&id.to_ne_bytes(), vertex.get_bytes().as_slice()].concat()
                        })
                        .collect::<Vec<_>>(),
                    outline_vertices
                        .into_iter()
                        .flat_map(move |vertex| {
                            [&id.to_ne_bytes(), vertex.get_bytes().as_slice()].concat()
                        })
                        .collect::<Vec<_>>(),
                )
            })
            .unzip();

        vec![fill.concat(), outline.concat()]
    }

    fn build_programs(&self, gl: &mut OpenGl) -> Vec<Rc<RefCell<Program>>> {
        let vertex_format = &[
            // ID
            VertexFormat::new(1, VertexType::UInt),
            // Vertex
            VertexFormat::new(3, VertexType::Float),
        ];

        [
            ("canvas_fill", DrawType::Triangles),
            ("canvas_outline", DrawType::LineStrip),
        ]
        .into_iter()
        .map(|(directory, draw_type)| {
            gl.add_program(
                Program::from_directory(directory)
                    .unwrap()
                    .with_format(vertex_format)
                    .with_draw_type(draw_type),
            )
            .unwrap()
        })
        .collect()
    }

    fn get_texture_buffer(&self) -> Option<Vec<u8>> {
        None
        // Some(
        // self.objects
        //     .iter()
        //     .flat_map(|object| {
        // // TODO: Pack meta bytes here (eg stroke enabled, fill enabled)

        // [
        // object
        //     .get_stroke_width()
        //     .unwrap_or_default()
        //     .to_ne_bytes()
        //     .as_slice(),
        // object
        //     .get_stroke_color()
        //     .unwrap_or_default()
        //     .get_bytes()
        //     .as_slice(),
        // object
        //     .get_stroke_dash()
        //     .unwrap_or_default()
        //     .to_ne_bytes()
        //     .as_slice(),
        // object.get_fill().unwrap_or_default().get_bytes().as_slice(),
        //     ]
        //     .concat()
        // })
        // .collect(),
        // )
    }
}

pub trait RenderStep {
    fn build_programs(&self, gl: &mut OpenGl) -> Vec<Rc<RefCell<Program>>>;
    fn get_vertices(&self) -> Vec<Vec<u8>>;
    fn get_texture_buffer(&self) -> Option<Vec<u8>> {
        None
    }
}

pub struct World<'a> {
    window: Window,
    projection: Mat4,
    camera: Camera,

    render_steps: Vec<Box<dyn RenderStep + 'a>>,

    lines: Vec<Line>,
    polygons: Vec<Polygon>,
}

impl<'a> World<'a> {
    pub fn with_window(window: Window) -> Self {
        let aspect_ratio = {
            let size = window.get_size();
            size.1 as f32 / size.0 as f32
        };

        Self {
            window,
            projection: Mat4::perspective_rh(PI / 2.0, aspect_ratio, 1.0, 50.0),
            camera: Camera::new(),
            render_steps: Vec::new(),
            lines: Vec::new(),
            polygons: Vec::new(),
        }
    }

    pub fn add_render_step<R>(&mut self, render_step: R)
    where
        R: RenderStep + 'a,
    {
        self.render_steps.push(Box::new(render_step));
    }

    pub fn add_line(&mut self, line: Line) {
        self.lines.push(line);
    }

    pub fn add_polygon(&mut self, polygon: Polygon) {
        self.polygons.push(polygon);
    }

    pub fn run(mut self) -> ! {
        let programs = self
            .render_steps
            .iter()
            .flat_map(|render_step| {
                let programs = render_step.build_programs(&mut self.window.gl);
                let vertices = render_step.get_vertices();

                for (program, vertices) in programs.iter().zip(vertices) {
                    // Attach vertices
                    let mut program = program.borrow_mut();
                    program.attach_vertices(vertices, None).unwrap();
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

            line_program
                .attach_vertices(self.lines.as_slice(), None)
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

                    // dbg!(plane_intersection);

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
