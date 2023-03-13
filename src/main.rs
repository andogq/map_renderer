mod objects;
mod osm;
mod renderer;

use clap::Parser;
use osm::Osm;
use osmpbf::ElementReader;
use renderer::render;
use softbuffer::GraphicsContext;
use winit::{
    dpi::{LogicalSize, PhysicalSize},
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
        .unwrap();

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

    // Prepare app state
    let mut app_state = {
        let PhysicalSize { height, width, .. } = window.inner_size();

        AppState {
            bounding,
            height,
            width,
        }
    };

    // Start event loop
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                // Update window size
                let (width, height) = {
                    let size = window.inner_size();
                    (size.width, size.height)
                };
                app_state.width = width;
                app_state.height = height;

                // Render data
                let dt = render(&app_state, &osm_data);

                // Push buffer to window
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
