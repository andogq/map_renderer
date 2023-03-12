mod objects;
mod osm;
mod renderable;

use osm::Osm;
use osmpbf::ElementReader;
use piet::{kurbo::PathEl, Color, RenderContext};
use piet_common::Device;
use renderable::Point;
use std::mem;

const SIZE: usize = 500;
const SCALE: usize = 8;

#[derive(Debug)]
struct Bounding {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

fn main() -> osmpbf::Result<()> {
    let reader = ElementReader::from_path("data/cbd.osm.pbf").expect("data/cbd.osm.pbf to exist");
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

    let mut device = Device::new().unwrap();
    let mut bitmap = device
        .bitmap_target(SIZE * SCALE, SIZE * SCALE, SCALE as f64)
        .unwrap();
    let mut ctx = bitmap.render_context();

    ctx.clear(None, Color::WHITE);

    for (_way_id, way) in osm_data.ways.iter() {
        if let Some(way_type) = way.get_way_type() {
            let points = way
                .nodes
                .iter()
                .filter_map(|node_id| osm_data.nodes.get(node_id))
                .map(|node| {
                    let x = ((node.x - bounding.min_x) / scaling) * (SIZE as f64);
                    let y = (1.0 - (node.y - bounding.min_y) / scaling) * (SIZE as f64);

                    Point::new(x, y)
                })
                .collect::<Vec<_>>();

            for renderable in way_type.get_renderables(&points) {
                let mut points = renderable.path.into_iter().map(|p| p.into());

                let path = &[
                    &[PathEl::MoveTo(points.next().unwrap())],
                    &points.map(PathEl::LineTo).collect::<Vec<_>>()[..],
                ]
                .concat();

                if let Some(fill) = renderable.fill {
                    ctx.fill(&path[..], &fill);
                }

                if let Some(stroke) = renderable.stroke {
                    ctx.stroke_styled(
                        &path[..],
                        &stroke.color,
                        stroke.width,
                        &stroke.get_piet_stroke_style(),
                    );
                }
            }
        }
    }

    ctx.finish().unwrap();
    mem::drop(ctx);
    bitmap.save_to_file("output.png").unwrap();

    Ok(())
}
