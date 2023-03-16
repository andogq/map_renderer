use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

const WINDOW_SIZE: (usize, usize) = (480, 720);

fn main() {
    let event_loop = EventLoop::new();
    let window = {
        let size = LogicalSize::new(WINDOW_SIZE.1 as f32, WINDOW_SIZE.0 as f32);
        WindowBuilder::new()
            .with_title("OpenGL Renderer")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    event_loop.run(move |event, _, control_flow| {
        control_flow.set_wait();

        #[allow(clippy::collapsible_match)]
        match event {
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                // Redraw window
                println!("Redraw requested");
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
                } =>
                {
                    #[allow(clippy::single_match)]
                    match keycode {
                        VirtualKeyCode::Escape => control_flow.set_exit(),
                        _ => {}
                    }
                }
                WindowEvent::CloseRequested => control_flow.set_exit(),
                _ => {}
            },
            _ => {}
        }
    })
}
