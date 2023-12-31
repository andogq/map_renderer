use std::f64::consts::PI;

use super::Tags;

const WGS84_A: f64 = 6378137.0;

pub struct Node {
    pub x: f64,
    pub y: f64,
    pub tags: Tags,
}

impl Node {
    #[allow(dead_code)]
    pub fn new(x: f64, y: f64) -> Node {
        Node {
            x,
            y,
            tags: Tags::new(),
        }
    }

    pub fn from_lon_lat(lon: f64, lat: f64) -> Node {
        // Lon degrees to radians
        let lon = lon * PI / 180.0;
        let lat = lat * PI / 180.0;

        let x = WGS84_A * (lon + (2.0 * PI));
        let y = WGS84_A * f64::ln((lat.sin() + 1.0) / lat.cos());

        Node {
            x,
            y,
            tags: Tags::new(),
        }
    }
}

impl From<osmpbf::Node<'_>> for Node {
    fn from(node: osmpbf::Node) -> Self {
        let mut n = Self::from_lon_lat(node.lon(), node.lat());
        n.tags = node.tags().into();

        n
    }
}

impl From<osmpbf::DenseNode<'_>> for Node {
    fn from(node: osmpbf::DenseNode) -> Self {
        let mut n = Self::from_lon_lat(node.lon(), node.lat());
        n.tags = node.tags().into();

        n
    }
}
