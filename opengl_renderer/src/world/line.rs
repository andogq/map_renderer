use glam::Vec3;

pub struct Line {
    pub points: Vec<Vec3>,
    pub width: f32,
    pub color: Vec3,
}
impl Line {
    pub fn flatten(&self) -> Vec<f32> {
        self.points
            .iter()
            .flat_map(|point| {
                [
                    point.to_array().as_slice(),
                    &[self.width],
                    self.color.to_array().as_slice(),
                ]
                .concat()
            })
            .collect()
    }
}
