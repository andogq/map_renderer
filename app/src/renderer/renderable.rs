use super::{Color, Point, Stroke};

#[derive(Debug)]
pub struct Renderable {
    pub path: Vec<Point>,
    pub stroke: Option<Stroke>,
    pub fill: Option<Color>,
}

impl Renderable {
    pub fn from_points(points: &[Point]) -> Self {
        Self {
            path: points.to_vec(),
            stroke: None,
            fill: None,
        }
    }

    pub fn with_stroke(mut self, stroke: Stroke) -> Self {
        self.stroke = Some(stroke);
        self
    }

    pub fn with_fill(mut self, fill: Color) -> Self {
        self.fill = Some(fill);
        self
    }
}
