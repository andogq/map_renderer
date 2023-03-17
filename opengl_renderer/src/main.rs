use crate::window::Window;
use crate::world::World;
use opengl::OpenGlError;

mod opengl;
mod window;
mod world;

const WINDOW_SIZE: (usize, usize) = (480, 720);

fn main() -> Result<(), OpenGlError> {
    let window = Window::new(WINDOW_SIZE);

    println!("{:?}", window.gl.get_info());

    let world = World::with_window(window);
    world.run()
}
