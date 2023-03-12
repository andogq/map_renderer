use super::Object;
use crate::renderable::{Point, Renderable, Stroke, StrokeStyle};
use piet::Color;

pub struct Park;

impl Object for Park {
    fn get_renderables(&self, points: &[Point]) -> Vec<Renderable> {
        vec![Renderable::from_points(points)
            .with_stroke(Stroke {
                width: 1.0,
                color: Color::rgb8(122, 175, 117),
                style: StrokeStyle::Solid,
            })
            .with_fill(Color::rgb8(205, 247, 201))]
    }
}
