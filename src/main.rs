mod objects;
mod osm;
mod renderable;

use clap::Parser;
use osm::Osm;
use osmpbf::ElementReader;
use raqote::{DrawOptions, DrawTarget, PathBuilder};
use renderable::Point;
use softbuffer::GraphicsContext;
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

#[derive(Parser)]
struct Args {
    /// Open Street Data PBF data file
    pbf_file: String,

    /// Minimum window size
    #[arg(long, default_value_t = 500)]
    size: usize,
}

#[derive(Debug)]
struct Bounding {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
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
        .unwrap();

    let d_lat = bounding.max_y - bounding.min_y;
    let d_lon = bounding.max_x - bounding.min_x;
    let scaling = f64::max(d_lat, d_lon);

    // Set up window
    let event_loop = EventLoop::new();
    let window = {
        let size = LogicalSize::new(args.size as f32, args.size as f32);
        WindowBuilder::new()
            .with_title("Map")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };
    let mut graphics_context = unsafe { GraphicsContext::new(&window, &window) }.unwrap();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                let (width, height) = {
                    let size = window.inner_size();
                    (size.width, size.height)
                };

                let size = u32::min(width, height);

                let mut dt = DrawTarget::new(width as i32, height as i32);
                dt.clear(raqote::SolidSource::from_unpremultiplied_argb(
                    0xff, 0xff, 0xff, 0xff,
                ));

                for (_way_id, way) in osm_data.ways.iter() {
                    if let Some(way_type) = way.to_object() {
                        let points = way
                            .nodes
                            .iter()
                            .filter_map(|node_id| osm_data.nodes.get(node_id))
                            .map(|node| {
                                let x = ((node.x - bounding.min_x) / scaling) * (size as f64);
                                let y = (1.0 - (node.y - bounding.min_y) / scaling) * (size as f64);

                                Point::new(x as f32, y as f32)
                            })
                            .collect::<Vec<_>>();

                        for renderable in way_type.get_renderables(&points) {
                            let mut points = renderable.path.into_iter();

                            let path = {
                                let p = points.next().unwrap();
                                let mut path = PathBuilder::new();
                                path.move_to(p.x, p.y);
                                path
                            };
                            let path = points
                                .fold(path, |mut path, p| {
                                    path.line_to(p.x, p.y);
                                    path
                                })
                                .finish();

                            if let Some(fill) = &renderable.fill {
                                dt.fill(&path, &fill.into(), &DrawOptions::new());
                            }

                            if let Some(stroke) = &renderable.stroke {
                                dt.stroke(
                                    &path,
                                    &(&stroke.color).into(),
                                    &stroke.into(),
                                    &DrawOptions::new(),
                                );
                            }
                        }
                    }
                }

                graphics_context.set_buffer(dt.get_data(), width as u16, height as u16);
            }
            Event::WindowEvent {
                window_id,
                event: WindowEvent::CloseRequested,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,
            _ => {}
        }
    });
}
