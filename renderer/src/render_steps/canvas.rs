use crate::{
    ogl::{DrawType, OpenGl, Program, VertexData, VertexFormat, VertexType},
    RenderStep,
};
use glam::Vec3;
use std::{cell::RefCell, rc::Rc};

#[derive(Default)]
pub struct Path {
    points: Vec<Vec3>,
    stroke: Option<Stroke>,
    fill: Option<Fill>,
}

impl Path {
    pub fn new(points: Vec<Vec3>) -> Self {
        Path {
            points,
            ..Path::default()
        }
    }

    pub fn with_stroke(mut self, stroke: Stroke) -> Self {
        self.stroke = Some(stroke);

        self
    }

    pub fn with_fill(mut self, fill: Vec3) -> Self {
        self.fill = Some(Fill::new(fill, &self.points));

        self
    }
}

impl CanvasObject for Path {
    fn get_vertices(&self) -> Vec<Vec3> {
        // TODO: Generate dashed segments here?

        self.points.clone()
    }

    fn get_stroke(&self) -> Option<Stroke> {
        self.stroke.clone()
    }

    fn get_fill(&self) -> Option<Fill> {
        self.fill.clone()
    }
}

#[derive(Clone, Debug, Default)]
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
    let triangle_area = (b - a).cross(c - a).length() / 2.0;
    let alpha = (b - p).cross(c - p).length() / (2.0 * triangle_area);
    let beta = (c - p).cross(a - p).length() / (2.0 * triangle_area);
    let gamma = (b - p).cross(a - p).length() / (2.0 * triangle_area);

    (0.0..=1.0).contains(&alpha)
        && (0.0..=1.0).contains(&beta)
        && (0.0..=1.0).contains(&gamma)
        && alpha + beta + gamma == 1.0
}

#[derive(Clone, Debug, Default)]
pub struct Fill {
    indexes: Vec<usize>,
    fill: Vec3,
}
impl Fill {
    pub fn new(fill: Vec3, outline: &[Vec3]) -> Self {
        // TODO: temp, rmeove duplicated end point
        let outline = &outline[0..outline.len() - 1];

        // Determine path direction (clockwise/anti-clockwise)
        let smallest = outline
            .iter()
            .enumerate()
            .reduce(|(smallest_i, smallest), (i, node)| {
                if node.z < smallest.z || (node.z == smallest.z && node.x < smallest.x) {
                    (i, node)
                } else {
                    (smallest_i, smallest)
                }
            })
            .unwrap();
        let left_side = outline[(smallest.0 + outline.len() - 1) % outline.len()] - *smallest.1;
        let right_side = outline[(smallest.0 + outline.len() + 1) % outline.len()] - *smallest.1;
        let direction = left_side.cross(right_side).length().signum();

        let mut indexes = Vec::new();
        let mut remaining_indexes = (0..outline.len()).collect::<Vec<_>>();

        let mut i = 0;
        'point_loop: while remaining_indexes.len() > 3 {
            let left_i = remaining_indexes[i % remaining_indexes.len()];
            let center_i = remaining_indexes[(i + 1) % remaining_indexes.len()];
            let right_i = remaining_indexes[(i + 2) % remaining_indexes.len()];

            let left = outline[left_i];
            let center = outline[center_i];
            let right = outline[right_i];

            // Check angle between center point
            let left_side = left - center;
            let right_side = right - center;

            // Assumes that polygon is on y=0 plane
            let cross = left_side.cross(right_side);

            if cross.length().signum() == direction {
                // Internal angle
                for &index in remaining_indexes.iter() {
                    let p = outline[index];
                    if p == left || p == center || p == right {
                        continue;
                    }

                    if point_in_triangle(p, left, center, right) {
                        i += 1;
                        continue 'point_loop;
                    }
                }

                // If reached here, everything is valid
                indexes.extend_from_slice({
                    let c = remaining_indexes.remove((i + 1) % remaining_indexes.len());

                    &[left_i, c, right_i]
                });

                continue;
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

#[derive(Default)]
pub struct CanvasProgram<'a> {
    objects: Vec<Box<dyn CanvasObject + 'a>>,
}

impl<'a> CanvasProgram<'a> {
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

                let outline_vertices = object.get_vertices();

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
        Some(
            self.objects
                .iter()
                .flat_map(|object| {
                    let stroke = object.get_stroke();
                    let fill = object.get_fill();

                    [
                        {
                            [stroke.is_some(), fill.is_some()]
                                .into_iter()
                                .enumerate()
                                .fold(0u32, |packed, (i, value)| (packed << 1) | (value as u32))
                                .to_ne_bytes()
                                .as_slice()
                        },
                        {
                            let stroke = stroke.unwrap_or_default();

                            [
                                stroke.color.get_bytes().as_slice(),
                                stroke.width.to_ne_bytes().as_slice(),
                                stroke.dash.unwrap_or_default().to_ne_bytes().as_slice(),
                            ]
                            .concat()
                            .as_slice()
                        },
                        {
                            let fill = fill.unwrap_or_default();

                            fill.fill.get_bytes().as_slice()
                        },
                    ]
                    .concat()
                })
                .collect(),
        )
    }
}
