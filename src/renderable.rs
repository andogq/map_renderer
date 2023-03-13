#[derive(Clone, Copy)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
}
impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}
impl From<&Color> for raqote::SolidSource {
    fn from(color: &Color) -> Self {
        Self::from_unpremultiplied_argb(0xff, color.r, color.g, color.b)
    }
}
impl From<&Color> for raqote::Source<'_> {
    fn from(color: &Color) -> Self {
        Self::Solid(color.into())
    }
}

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

#[allow(dead_code)]
pub enum DashStyle {
    Even,
    Dot,
    Custom(Vec<f32>),
}

impl DashStyle {
    fn get_pattern(&self) -> Vec<f32> {
        match self {
            DashStyle::Even => vec![1.0, 1.0],
            DashStyle::Dot => vec![0.5, 2.0],
            DashStyle::Custom(dash) => dash.clone(),
        }
    }
}

pub enum StrokeStyle {
    Solid,
    Dashed(DashStyle),
    Doubled {
        outer_width: f32,
        outer_color: Color,
    },
}

impl StrokeStyle {
    pub fn get_dash_array(&self) -> Option<Vec<f32>> {
        match self {
            Self::Dashed(dashed_style) => Some(dashed_style.get_pattern()),
            _ => None,
        }
    }
}

pub struct Stroke {
    pub width: f32,
    pub color: Color,
    pub style: StrokeStyle,
}
impl From<&Stroke> for raqote::StrokeStyle {
    fn from(stroke: &Stroke) -> Self {
        raqote::StrokeStyle {
            width: stroke.width,
            cap: raqote::LineCap::Round,
            join: raqote::LineJoin::Round,
            miter_limit: 4.0, // Default according to MDN
            dash_array: stroke.style.get_dash_array().unwrap_or_default(),
            dash_offset: 0.0,
        }
    }
}

pub struct Renderable {
    pub path: Vec<Point>,
    pub stroke: Option<Stroke>,
    pub fill: Option<Color>,
}

impl Renderable {
    pub fn from_points(points: &[Point]) -> Self {
        Self {
            path: points.to_vec(),
            stroke: None,
            fill: None,
        }
    }

    pub fn with_stroke(mut self, stroke: Stroke) -> Self {
        self.stroke = Some(stroke);
        self
    }

    pub fn with_fill(mut self, fill: Color) -> Self {
        self.fill = Some(fill);
        self
    }
}
