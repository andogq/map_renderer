use crate::{osm::Osm, Point};

#[derive(Debug)]
pub(crate) struct Bounding {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}
impl Bounding {
    pub fn dx(&self) -> f64 {
        (self.max_x - self.min_x).abs()
    }

    pub fn dy(&self) -> f64 {
        (self.max_y - self.min_y).abs()
    }

    pub fn equalise(mut self) -> Self {
        let largest = f64::max(self.dy(), self.dx());

        let dy = largest - self.dy();
        self.min_y -= dy / 2.0;
        self.max_y += dy / 2.0;

        let dx = largest - self.dx();
        self.min_x -= dx / 2.0;
        self.max_x += dx / 2.0;

        self
    }
}

pub(crate) struct MapData {
    bounding: Bounding,
    pub osm_data: Osm,
    scaling: f64,
}

impl MapData {
    pub fn new(osm_data: Osm, scaling_factor: f64) -> Self {
        let bounding = osm_data
            .nodes
            .values()
            .fold(None::<Bounding>, |bounding, node| {
                Some(if let Some(bounding) = &bounding {
                    Bounding {
                        min_x: bounding.min_x.min(node.x),
                        min_y: bounding.min_y.min(node.y),
                        max_x: bounding.max_x.max(node.x),
                        max_y: bounding.max_y.max(node.y),
                    }
                } else {
                    Bounding {
                        min_x: node.x,
                        min_y: node.y,
                        max_x: node.x,
                        max_y: node.y,
                    }
                })
            })
            .unwrap()
            .equalise();

        let scaling = scaling_factor / f64::max(bounding.dx(), bounding.dy());

        Self {
            bounding,
            osm_data,
            scaling,
        }
    }

    pub fn scale(&self, p: Point) -> Point {
        // TODO: Fix up mismatched types
        let x = (p.x as f64 - self.bounding.min_x - (self.bounding.dx() / 2.0)) * self.scaling;
        let y = (p.y as f64 - self.bounding.min_y - (self.bounding.dy() / 2.0)) * self.scaling;

        Point::new(x as f32, y as f32)
    }
}
