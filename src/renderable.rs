use piet::Color;

#[derive(Clone)]
pub struct Point {
    x: f64,
    y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Point { x, y }
    }
}

impl From<Point> for piet::kurbo::Point {
    fn from(value: Point) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

#[allow(dead_code)]
pub enum DashStyle {
    Even,
    Dot,
    Custom(&'static [f64]),
}

impl DashStyle {
    fn get_pattern(&self) -> &'static [f64] {
        match self {
            DashStyle::Even => &[1.0, 1.0],
            DashStyle::Dot => &[0.5, 2.0],
            DashStyle::Custom(dash) => dash,
        }
    }
}

pub enum StrokeStyle {
    Solid,
    Dashed(DashStyle),
    Doubled {
        outer_width: f64,
        outer_color: Color,
    },
}

impl StrokeStyle {
    pub fn get_piet_stroke_dash(&self) -> Option<&'static [f64]> {
        match self {
            Self::Dashed(dashed_style) => Some(dashed_style.get_pattern()),
            _ => None,
        }
    }
}

pub struct Stroke {
    pub width: f64,
    pub color: Color,
    pub style: StrokeStyle,
}
impl Stroke {
    pub fn get_piet_stroke_style(&self) -> piet::StrokeStyle {
        let mut style = piet::StrokeStyle::new()
            .line_join(piet::LineJoin::Round)
            .line_cap(piet::LineCap::Round);

        if let Some(dash) = self.style.get_piet_stroke_dash() {
            style = style.dash_pattern(dash);
        }

        style
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

