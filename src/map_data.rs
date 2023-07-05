use crate::{osm::Osm, Point};

#[derive(Debug)]
pub(crate) struct Bounding {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
    pub center_x: f64,
    pub center_y: f64,
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
    pub bounding: Bounding,
    pub osm_data: Osm,
}

impl MapData {
    pub fn new(osm_data: Osm) -> Self {
        let mut bounding = osm_data
            .nodes
            .values()
            .fold(None::<Bounding>, |bounding, node| {
                Some(if let Some(bounding) = &bounding {
                    Bounding {
                        min_x: bounding.min_x.min(node.x),
                        min_y: bounding.min_y.min(node.y),
                        max_x: bounding.max_x.max(node.x),
                        max_y: bounding.max_y.max(node.y),
                        center_x: 0.0,
                        center_y: 0.0,
                    }
                } else {
                    Bounding {
                        min_x: node.x,
                        min_y: node.y,
                        max_x: node.x,
                        max_y: node.y,
                        center_x: 0.0,
                        center_y: 0.0,
                    }
                })
            })
            .unwrap()
            .equalise();

        bounding.center_x = (bounding.min_x + bounding.max_x) / 2.0;
        bounding.center_y = (bounding.min_y + bounding.max_y) / 2.0;

        Self { bounding, osm_data }
    }

    pub fn translate(&self, point: Point) -> Point {
        // Translate data back to center
        Point::new(
            point.x - (self.bounding.center_x as f32),
            point.y - (self.bounding.center_y as f32),
        )
    }
}
