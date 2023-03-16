use super::Object;
use crate::renderer::{Color, Point, Renderable};

pub struct Building;

impl Object for Building {
    fn get_renderables(&self, points: &[Point]) -> Vec<Renderable> {
        vec![Renderable::from_points(points).with_fill(Color::new(0xcc, 0xcc, 0xcc))]
    }
}