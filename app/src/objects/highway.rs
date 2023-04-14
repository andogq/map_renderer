use glam::Vec3;
use renderer::render_steps::canvas::{Path, Stroke};

use super::Object;
use crate::{osm::Tags, Point};

// https://wiki.openstreetmap.org/wiki/Key:highway?uselang=en-GB
#[allow(dead_code)]
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
    pub fn from_tags(tags: &Tags) -> Option<Highway> {
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
    fn get_paths(&self, points: &[Point]) -> Vec<Path> {
        vec![
            Path::new(points.iter().map(|p| p.into()).collect()).with_stroke({
                let width = match self {
                    Self::Motorway => 4.0,
                    Self::Trunk | Self::Primary | Self::Secondary | Self::Tertiary => 2.0,
                    Self::Service => 0.75,
                    Self::Footway | Self::Path => 0.5,
                    _ => 1.0,
                };
                let color = match self {
                    Self::Motorway => Vec3::new(223.0, 46.0, 107.0) / 255.0,
                    Self::Trunk => Vec3::new(234.0, 144.0, 161.0) / 255.0,
                    Self::Primary => Vec3::new(252.0, 192.0, 171.0) / 255.0,
                    Self::Secondary => Vec3::new(253.0, 214.0, 1.0) / 255.0,
                    Self::Tertiary => Vec3::new(246.0, 250.0, 187.0) / 255.0,
                    Self::Footway | Self::Path => Vec3::new(250.0, 164.0, 156.0) / 255.0,
                    _ => Vec3::new(169.0, 175.0, 182.0) / 255.0,
                };
                let dash = match self {
                    Self::Footway | Self::Path => Some(0.2),
                    Self::Motorway
                    | Self::Trunk
                    | Self::Primary
                    | Self::Secondary
                    | Self::Tertiary => Some(0.5),
                    _ => None,
                };

                let mut stroke = Stroke::new(width, color);

                if let Some(dash) = dash {
                    stroke = stroke.with_dash(dash);
                }

                stroke
            }),
        ]
    }
}
