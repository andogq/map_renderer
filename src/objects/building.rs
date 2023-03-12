use super::Object;
use crate::renderable::{Point, Renderable};
use piet::Color;

pub struct Building;

impl Object for Building {
    fn get_renderables(&self, points: &[Point]) -> Vec<Renderable> {
        vec![Renderable::from_points(points).with_fill(Color::GRAY)]
    }
}
