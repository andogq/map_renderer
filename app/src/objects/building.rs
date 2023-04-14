use super::Object;
use crate::Point;
use glam::Vec3;
use renderer::render_steps::canvas::Path;

pub struct Building;

impl Object for Building {
    fn get_paths(&self, points: &[Point]) -> Vec<Path> {
        vec![Path::new(points.iter().map(|p| p.into()).collect())
            .with_fill(Vec3::new(0.7, 0.7, 0.7))]
    }
}
