use renderer::window::Window;
use renderer::Renderer;

const WINDOW_SIZE: (usize, usize) = (480, 720);

fn main() -> Result<(), opengl::OpenGlError> {
    let window = Window::new(WINDOW_SIZE);

    println!("{:?}", window.gl.get_info());

    let renderer = Renderer::with_window(window);

    renderer.run();
}
