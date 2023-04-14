use super::Object;
use crate::Point;
use glam::Vec3;
use renderer::render_steps::canvas::{Path, Stroke};

pub struct Railway;

impl Object for Railway {
    fn get_paths(&self, points: &[Point]) -> Vec<Path> {
        vec![Path::new(points.iter().map(|p| p.into()).collect())
            .with_stroke(Stroke::new(0.1, Vec3::new(164.0, 214.0, 255.0) / 255.0))]
    }
}
