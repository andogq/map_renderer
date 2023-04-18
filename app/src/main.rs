mod map_canvas;
mod map_data;
mod objects;
mod osm;
mod path_finder;
mod plugin;

use std::rc::Rc;

use clap::Parser;
use glam::Vec3;
use map_canvas::MapCanvas;
use map_data::MapData;
use osm::{Node, Osm};
use osmpbf::ElementReader;
use path_finder::PathFinder;
use plugin::Plugin;
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

fn main() -> osmpbf::Result<()> {
    let args = Args::parse();

    // Load plugins
    let mut plugins: [Box<dyn Plugin<()>>; 1] = [Box::new(MapCanvas::new())];

    // Load map data from disk
    let reader = ElementReader::from_path(&args.pbf_file).expect("input file should exist");
    let osm_data = Osm::from_reader(reader)?;
    let map_data = Rc::new(MapData::new(osm_data, 500.0));

    // Initialise window and renderer
    let window = Window::new((args.size, args.size));
    let mut renderer = Renderer::with_window(window);

    for plugin in plugins.iter_mut() {
        plugin.with_map_data(Rc::clone(&map_data));
        renderer.add_render_step(plugin.get_render_step());
    }

    renderer.run(move |event| {
        for plugin in plugins.iter_mut() {
            plugin.handle_event((), event);
        }
    });
}
