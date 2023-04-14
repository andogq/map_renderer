use super::Object;
use crate::Point;
use glam::Vec3;
use renderer::render_steps::canvas::{Path, Stroke};

pub struct Park;

impl Object for Park {
    fn get_paths(&self, points: &[Point]) -> Vec<Path> {
        vec![Path::new(points.iter().map(|p| p.into()).collect())
            .with_fill(Vec3::new(205.0, 247.0, 201.0) / 255.0)
            .with_stroke(Stroke::new(1.0, Vec3::new(122.0, 175.0, 117.0) / 255.0))]
    }
}
