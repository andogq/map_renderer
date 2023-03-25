use crate::window::Window;
use crate::world::line::Line;
use crate::world::World;
use glam::Vec3;
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
        points: vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 10.0),
            Vec3::new(10.0, 0.0, 10.0),
            Vec3::new(10.0, 0.0, 0.0),
        ],
        width: 1.0,
        color: Vec3::new(0.3, 0.8, 0.1),
        stroke_length: Some(2.0),
    });

    world.add_line(Line {
        points: vec![Vec3::new(-5.0, 0.0, 3.0), Vec3::new(-9.0, 0.0, 7.0)],
        width: 3.0,
        color: Vec3::new(0.7, 0.2, 0.4),
        stroke_length: None,
    });

    println!("{:?}", (true as u32).to_ne_bytes());

    world.run();
}
