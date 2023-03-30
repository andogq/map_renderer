use glam::Vec3;

use crate::opengl::VertexData;

use super::CanvasObject;

pub struct Polygon {
    points: Vec<Vec3>,
    color: Vec3,

    triangle_cache: Option<Vec<Vec3>>,
}

pub fn point_in_triangle(p: &Vec3, a: &Vec3, b: &Vec3, c: &Vec3) -> bool {
    // https://www.youtube.com/watch?v=HYAgJN3x4GA
    let w1 = ((a.x * (c.z - a.z)) + ((p.z - a.z) * (c.x - a.x)) - (p.x * (c.z - a.z)))
        / (((b.z - a.z) * (b.x - a.x)) - ((b.x - a.x) * (c.z - a.z)));
    let w2 = (p.z - a.z - (w1 * (b.z - a.z))) / (c.z - a.z);

    w1 >= 0.0 && w2 >= 0.0 && (w1 + w2) <= 1.0
}

impl Polygon {
    pub fn new(points: Vec<Vec3>, color: Vec3) -> Self {
        Self {
            points,
            color,
            triangle_cache: None,
        }
    }
    pub fn triangulate(&mut self) -> Vec<Vec3> {
        if let Some(points) = &self.triangle_cache {
            points.clone()
        } else {
            let mut points = Vec::new();
            let mut remaining_points = self.points.clone();

            let mut i = 0;
            'point_loop: while remaining_points.len() > 3 {
                println!("{i}, {}", remaining_points.len());
                let left = remaining_points[i % remaining_points.len()];
                let center = remaining_points[(i + 1) % remaining_points.len()];
                let right = remaining_points[(i + 2) % remaining_points.len()];

                // Check angle between center point
                let left_side = left - center;
                let right_side = right - center;

                // Assumes that polygon is on y=0 plane
                let cross = left_side.cross(right_side);

                if cross.y < 0.0 {
                    // Internal angle
                    println!("pass");

                    for p in remaining_points.iter() {
                        if p == &left || p == &center || p == &right {
                            continue;
                        }

                        if point_in_triangle(p, &center, &left, &right) {
                            i += 1;
                            continue 'point_loop;
                        }
                    }

                    // If reached here, everything is valid
                    points.extend_from_slice(&[
                        left,
                        remaining_points.remove((i + 1) % remaining_points.len()),
                        right,
                    ]);
                } else {
                    // External angle, skip
                    println!("fail");
                }

                i += 1;
            }

            points.extend(remaining_points.into_iter());

            self.triangle_cache = Some(points.clone());

            points
        }
    }
}

impl VertexData for Polygon {
    fn get_bytes(&self) -> Vec<u8> {
        self.triangle_cache
            .clone()
            .unwrap()
            .iter()
            .flat_map(|point| {
                [
                    point.get_bytes().as_slice(),
                    self.color.get_bytes().as_slice(),
                ]
                .concat()
            })
            .collect()
    }
}

impl CanvasObject for Polygon {
    fn get_vertices(&self) -> Vec<Vec3> {
        todo!()
    }

    fn get_stroke_color(&self) -> Option<Vec3> {
        todo!()
    }

    fn get_stroke_dash(&self) -> Option<f32> {
        todo!()
    }

    fn get_fill(&self) -> Option<Vec3> {
        todo!()
    }

    fn get_stroke_width(&self) -> Option<f32> {
        todo!()
    }
}
