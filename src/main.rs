mod objects;
mod osm;
mod renderer;

use clap::Parser;
use osm::{Node, Osm};
use osmpbf::ElementReader;
use renderer::render;
use softbuffer::GraphicsContext;
use winit::{
    dpi::{LogicalSize, PhysicalSize},
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::EventLoop,
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

pub enum ZoomDirection {
    In,
    Out,
}
pub enum PanDirection {
    Left,
    Right,
    Up,
    Down,
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
        control_flow.set_wait();

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
            Event::WindowEvent { window_id, event } if window_id == window.id() => match event {
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(keycode),
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                } => match keycode {
                    VirtualKeyCode::Equals => {
                        app_state.bounding.zoom(ZoomDirection::In);
                        window.request_redraw();
                    }
                    VirtualKeyCode::Minus => {
                        app_state.bounding.zoom(ZoomDirection::Out);
                        window.request_redraw();
                    }
                    VirtualKeyCode::Left | VirtualKeyCode::A => {
                        app_state.bounding.pan(PanDirection::Left);
                        window.request_redraw();
                    }
                    VirtualKeyCode::Right | VirtualKeyCode::D => {
                        app_state.bounding.pan(PanDirection::Right);
                        window.request_redraw();
                    }
                    VirtualKeyCode::Up | VirtualKeyCode::W => {
                        app_state.bounding.pan(PanDirection::Up);
                        window.request_redraw();
                    }
                    VirtualKeyCode::Down | VirtualKeyCode::S => {
                        app_state.bounding.pan(PanDirection::Down);
                        window.request_redraw();
                    }
                    VirtualKeyCode::Escape => control_flow.set_exit(),
                    _ => {}
                },
                WindowEvent::CloseRequested => control_flow.set_exit(),
                _ => {}
            },
            _ => {}
        }
    });
}
