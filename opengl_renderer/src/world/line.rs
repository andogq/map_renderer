use glam::Vec3;

use crate::opengl::VertexData;

use super::CanvasObject;

impl VertexData for Vec3 {
    fn get_bytes(&self) -> Vec<u8> {
        self.to_array()
            .iter()
            .flat_map(|n| n.to_ne_bytes())
            .collect()
    }
}

pub struct Line {
    pub id: u32,
    pub points: Vec<Vec3>,
    pub width: f32,
    pub color: Vec3,
    pub stroke_length: Option<f32>,
}

impl VertexData for Line {
    fn get_bytes(&self) -> Vec<u8> {
        self.points
            .iter()
            .flat_map(|point| {
                [
                    self.id.to_ne_bytes().as_slice(),
                    point.get_bytes().as_slice(),
                    self.width.to_ne_bytes().as_slice(),
                    self.color.get_bytes().as_slice(),
                    self.stroke_length
                        .unwrap_or_default()
                        .to_ne_bytes()
                        .as_slice(),
                ]
                .concat()
            })
            .collect()
    }
}

impl CanvasObject for Line {
    fn get_vertices(&self) -> Vec<Vec3> {
        self.points.clone()
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
