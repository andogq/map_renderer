use super::Object;
use crate::renderer::{Color, DashStyle, Point, Renderable, Stroke, StrokeStyle};

pub struct Railway;

impl Object for Railway {
    fn get_renderables(&self, points: &[Point]) -> Vec<Renderable> {
        let color = Color::new(164, 214, 255);

        vec![Renderable::from_points(points).with_stroke(Stroke {
            width: 1.0,
            color,
            style: StrokeStyle::Solid,
        })]
    }
}
