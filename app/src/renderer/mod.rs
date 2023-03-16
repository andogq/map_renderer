mod color;
mod point;
mod renderable;
mod stroke;

pub use color::*;
pub use point::*;
pub use renderable::*;
pub use stroke::*;

use crate::{osm::Osm, AppState};
use raqote::{DrawOptions, DrawTarget, PathBuilder};

pub(crate) fn render(
    AppState {
        bounding,
        height,
        width,
    }: &AppState,
    osm_data: &Osm,
) -> DrawTarget {
    let (width, height) = (*width, *height);
    let mut dt = DrawTarget::new(width as i32, height as i32);

    let d_lat = bounding.max_y - bounding.min_y;
    let d_lon = bounding.max_x - bounding.min_x;
    let scaling = (u32::min(height, width) as f64) / f64::max(d_lat, d_lon);

    dt.clear(raqote::SolidSource::from_unpremultiplied_argb(
        0xff, 0xff, 0xff, 0xff,
    ));

    let filtered_ways = osm_data.ways.values().filter(|way| {
        way.nodes
            .iter()
            .any(|node_id| bounding.contains(osm_data.nodes.get(node_id).unwrap()))
    });

    for way in filtered_ways {
        if let Some(way_type) = way.to_object() {
            let points = way
                .nodes
                .iter()
                .filter_map(|node_id| osm_data.nodes.get(node_id))
                .map(|node| {
                    let x = (node.x - bounding.min_x) * scaling;
                    let y = (height as f64) - ((node.y - bounding.min_y) * scaling);

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

    dt
}
