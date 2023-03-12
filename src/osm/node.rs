use std::f64::consts::PI;

use super::Tags;

pub struct Node {
    pub x: f64,
    pub y: f64,
    pub tags: Tags,
}

impl Node {
    pub fn new(x: f64, y: f64) -> Node {
        Node {
            x,
            y,
            tags: Tags::new(),
        }
    }

    pub fn from_lon_lat(lon: f64, lat: f64) -> Node {
        let x = lon;
        let y = f64::ln(f64::tan((PI / 4.0) + (lat / 2.0)));
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
