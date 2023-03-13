use super::Color;

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
