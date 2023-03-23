use glam::Vec3;

pub struct Line {
    pub start: Vec3,
    pub end: Vec3,
    pub width: f32,
    pub color: Vec3,
}
impl Line {
    pub fn flatten(&self) -> Vec<f32> {
        [
            self.start.to_array().as_slice(),
            &[self.width],
            self.color.to_array().as_slice(),
            self.end.to_array().as_slice(),
            &[self.width],
            self.color.to_array().as_slice(),
        ]
        .concat()
    }
}
