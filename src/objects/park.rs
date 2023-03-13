use super::Object;
use crate::renderable::{Color, Point, Renderable, Stroke, StrokeStyle};

pub struct Park;

impl Object for Park {
    fn get_renderables(&self, points: &[Point]) -> Vec<Renderable> {
        vec![Renderable::from_points(points)
            .with_stroke(Stroke {
                width: 1.0,
                color: Color::new(122, 175, 117),
                style: StrokeStyle::Solid,
            })
            .with_fill(Color::new(205, 247, 201))]
    }
}
