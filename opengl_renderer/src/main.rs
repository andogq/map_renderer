use crate::window::Window;
use crate::world::line::Line;
use crate::world::polygon::Polygon;
use crate::world::World;
use glam::Vec3;

mod opengl;
mod window;
mod world;

const WINDOW_SIZE: (usize, usize) = (480, 720);

fn main() -> Result<(), opengl::OpenGlError> {
    let window = Window::new(WINDOW_SIZE);

    println!("{:?}", window.gl.get_info());

    let mut world = World::with_window(window);
    world.add_line(Line {
        id: 0,
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
        id: 1,
        points: vec![Vec3::new(-5.0, 0.0, 3.0), Vec3::new(-9.0, 0.0, 7.0)],
        width: 3.0,
        color: Vec3::new(0.7, 0.2, 0.4),
        stroke_length: None,
    });

    let polygon_points = vec![
        Vec3::new(-3.0, 0.0, 3.0),
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(2.0, 0.0, 3.0),
        Vec3::new(3.0, 0.0, 1.0),
        Vec3::new(1.0, 0.0, -3.0),
    ];

    world.add_line(Line {
        id: 2,
        points: polygon_points
            .iter()
            .map(|p| *p + Vec3::new(5.0, 0.0, 0.0))
            .collect(),
        width: 0.3,
        color: Vec3::new(1.0, 0.3, 0.8),
        stroke_length: None,
    });

    let polygon = Polygon::new(polygon_points, Vec3::new(0.5, 0.3, 0.1));
    world.add_polygon(polygon);

    world.run();
}
