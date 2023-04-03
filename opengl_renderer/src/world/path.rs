use super::{CanvasObject, Fill, Stroke};
use glam::Vec3;

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
        self.points.clone()
    }

    fn get_stroke(&self) -> Option<Stroke> {
        self.stroke.clone()
    }

    fn get_fill(&self) -> Option<Fill> {
        self.fill.clone()
    }
}
