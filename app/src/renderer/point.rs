#[derive(Clone, Debug)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Point { x, y }
    }
}

impl From<Point> for raqote::Point {
    fn from(value: Point) -> Self {
        Self::new(value.x, value.y)
    }
}
