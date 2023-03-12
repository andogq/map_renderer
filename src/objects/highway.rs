use super::Object;
use crate::renderable::{DashStyle, Point, Renderable, Stroke, StrokeStyle};
use piet::Color;
use std::collections::HashMap;

// https://wiki.openstreetmap.org/wiki/Key:highway?uselang=en-GB
pub enum Highway {
    Motorway,
    Trunk,
    Primary,
    Secondary,
    Tertiary,
    Unclassified,
    Residential,
    Service,
    Footway,
    Path,
    Cycleway,
    Other,
}

impl Highway {
    pub fn from_tags(tags: &HashMap<String, String>) -> Option<Highway> {
        tags.get("highway").map(|tag| match tag.as_str() {
            "tertiary" => Highway::Tertiary,
            "residential" => Highway::Residential,
            "service" => Highway::Service,
            "footway" => Highway::Footway,
            "path" => Highway::Path,
            "cycleway" => Highway::Cycleway,
            _ => Highway::Other,
        })
    }
}

impl Object for Highway {
    fn get_renderables(&self, points: &[Point]) -> Vec<Renderable> {
        vec![Renderable::from_points(points).with_stroke({
            let width = match self {
                Self::Motorway => 4.0,
                Self::Trunk | Self::Primary | Self::Secondary | Self::Tertiary => 2.0,
                Self::Service => 0.75,
                Self::Footway | Self::Path => 0.5,
                _ => 1.0,
            };
            let color = match self {
                Self::Motorway => Color::rgb8(223, 46, 107),
                Self::Trunk => Color::rgb8(234, 144, 161),
                Self::Primary => Color::rgb8(252, 192, 171),
                Self::Secondary => Color::rgb8(253, 214, 1),
                Self::Tertiary => Color::rgb8(246, 250, 187),
                Self::Footway | Self::Path => Color::rgb8(250, 164, 156),
                _ => Color::rgb8(169, 175, 182),
            };
            let style = match self {
                Self::Footway | Self::Path => StrokeStyle::Dashed(DashStyle::Dot),
                Self::Motorway | Self::Trunk | Self::Primary | Self::Secondary | Self::Tertiary => {
                    StrokeStyle::Doubled {
                        outer_width: 0.5,
                        outer_color: Color::BLACK,
                    }
                }
                _ => StrokeStyle::Solid,
            };

            Stroke {
                width,
                color,
                style,
            }
        })]
    }
}
