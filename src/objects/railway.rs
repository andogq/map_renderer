use super::Object;
use crate::renderable::{DashStyle, Point, Renderable, Stroke, StrokeStyle};
use piet::Color;

pub struct Railway;

impl Object for Railway {
    fn get_renderables(&self, points: &[Point]) -> Vec<Renderable> {
        let color = Color::rgb8(164, 214, 255);

        vec![
            Renderable::from_points(points).with_stroke(Stroke {
                width: 1.0,
                color,
                style: StrokeStyle::Solid,
            }),
            Renderable::from_points(points).with_stroke(Stroke {
                width: 0.05,
                color,
                style: StrokeStyle::Dashed(DashStyle::Custom(&[0.1, 2.0])),
            }),
        ]
    }
}
