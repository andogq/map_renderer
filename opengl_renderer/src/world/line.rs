pub struct Point {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
impl Point {
    pub fn flatten(&self) -> [f32; 3] {
        [self.x, self.y, self.z]
    }
}

pub struct Line {
    pub start: Point,
    pub end: Point,
    pub width: f32,
}
impl Line {
    pub fn flatten(&self) -> Vec<f32> {
        [
            self.start.flatten().as_slice(),
            &[self.width],
            self.end.flatten().as_slice(),
            &[self.width],
        ]
        .concat()
    }
}
