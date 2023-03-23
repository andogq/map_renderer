use crate::window::Window;
use crate::world::line::{Line, Point};
use crate::world::World;
use opengl::OpenGlError;

mod opengl;
mod window;
mod world;

const WINDOW_SIZE: (usize, usize) = (480, 720);

fn main() -> Result<(), OpenGlError> {
    let window = Window::new(WINDOW_SIZE);

    println!("{:?}", window.gl.get_info());

    let mut world = World::with_window(window);
    world.add_line(Line {
        start: Point {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        end: Point {
            x: 10.0,
            y: 0.0,
            z: 10.0,
        },
        width: 1.0,
    });

    world.add_line(Line {
        start: Point {
            x: -5.0,
            y: 0.0,
            z: 3.0,
        },
        end: Point {
            x: -9.0,
            y: 0.0,
            z: 7.0,
        },
        width: 1.0,
    });

    world.run();
}
