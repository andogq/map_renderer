mod objects;
mod osm;

use clap::Parser;
use glam::Vec3;
use osm::{Node, Osm};
use osmpbf::ElementReader;
use renderer::{
    render_steps::canvas::{point_in_triangle, CanvasProgram, Path, Stroke},
    window::Window,
    Renderer,
};

#[derive(Parser)]
struct Args {
    /// Open Street Data PBF data file
    pbf_file: String,

    /// Minimum window size
    #[arg(long, default_value_t = 500)]
    size: usize,
}

enum ZoomDirection {
    In,
    Out,
}
enum PanDirection {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Clone, Copy)]
struct Point {
    x: f32,
    y: f32,
}
impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}
impl From<&Point> for Vec3 {
    fn from(value: &Point) -> Self {
        Vec3::new(value.x, 0.0, value.y)
    }
}

#[derive(Debug)]
struct Bounding {
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

    pub fn zoom(&mut self, direction: ZoomDirection) {
        let scale = 0.1
            * if let ZoomDirection::In = direction {
                1.0
            } else {
                -1.0
            };

        let dx = self.dx() * scale;
        let dy = self.dy() * scale;

        self.min_x += dx;
        self.max_x -= dx;
        self.min_y += dy;
        self.max_y -= dy;
    }

    pub fn pan(&mut self, direction: PanDirection) {
        let (dx, dy) = match direction {
            PanDirection::Left => (-1., 0.),
            PanDirection::Right => (1., 0.),
            PanDirection::Up => (0., 1.),
            PanDirection::Down => (0., -1.),
        };

        let scale = 0.05;
        let dx = dx * self.dx() * scale;
        let dy = dy * self.dy() * scale;

        self.min_x += dx;
        self.max_x += dx;
        self.min_y += dy;
        self.max_y += dy;
    }

    pub fn contains(&self, node: &Node) -> bool {
        node.x >= self.min_x && node.x <= self.max_x && node.y >= self.min_y && node.y <= self.max_y
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

struct AppState {
    pub bounding: Bounding,
    pub height: u32,
    pub width: u32,
}

fn main() -> osmpbf::Result<()> {
    let args = Args::parse();

    let reader = ElementReader::from_path(&args.pbf_file).expect("input file should exist");
    let osm_data = Osm::from_reader(reader)?;

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

    let window = Window::new((args.size, args.size));

    let mut world = Renderer::with_window(window);

    let d_lat = bounding.max_y - bounding.min_y;
    let d_lon = bounding.max_x - bounding.min_x;
    let scaling = 500_f64 / f64::max(d_lat, d_lon);

    let mut canvas = CanvasProgram::default();

    // Add all of the lines
    for (id, way) in osm_data.ways {
        if let Some(way_type) = way.to_object() {
            let points = way
                .nodes
                .iter()
                .filter_map(|node_id| osm_data.nodes.get(node_id))
                .map(|node| {
                    let x = (node.x - bounding.min_x - (d_lat / 2.0)) * scaling;
                    let y = (node.y - bounding.min_y - (d_lon / 2.0)) * scaling;

                    Point::new(x as f32, y as f32)
                })
                .collect::<Vec<_>>();

            for path in way_type.get_paths(&points) {
                canvas.add_object(path);
            }
        }
    }

    world.add_render_step(canvas);
    world.run();
}
