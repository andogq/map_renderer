use std::{cell::RefCell, rc::Rc};

use glam::Vec3;
use renderer::{
    render_steps::canvas::{CanvasProgram, Path},
    Event, RenderStep,
};

use crate::{map_data::MapData, plugin::Plugin, Point};

pub(crate) struct MapCanvas {
    canvas: Rc<RefCell<CanvasProgram>>,
    map_data: Option<Rc<MapData>>,
}

impl MapCanvas {
    pub fn new() -> Self {
        Self {
            canvas: Rc::new(RefCell::new(CanvasProgram::default())),
            map_data: None,
        }
    }
}

impl Plugin<()> for MapCanvas {
    fn with_map_data(&mut self, map_data: Rc<MapData>) {
        let mut canvas = self.canvas.borrow_mut();

        // Update canvas to re-draw all of the map elements
        canvas.clear();

        for way in map_data.osm_data.ways.values() {
            if let Some(way_type) = way.to_object() {
                let points = way
                    .nodes
                    .iter()
                    .filter_map(|node_id| map_data.osm_data.nodes.get(node_id))
                    .map(|node| Point::new(node.x as f32, node.y as f32))
                    .map(|node| map_data.translate(node))
                    .collect::<Vec<_>>();

                for path in way_type.get_paths(&points) {
                    canvas.add_object(Box::new(path));
                }
            }
        }

        // Save map data
        self.map_data = Some(map_data);
    }

    fn get_render_step(&self) -> Rc<RefCell<dyn RenderStep>> {
        Rc::clone(&self.canvas) as Rc<RefCell<dyn RenderStep>>
    }

    fn handle_event(&mut self, app_state: (), event: Event) -> bool {
        let mut canvas = self.canvas.borrow_mut();

        if let Event::Click(point) = event {
            canvas.add_object(Box::new(
                Path::new(
                    [
                        Vec3::new(0.0, 0.0, 0.0),
                        Vec3::new(20.0, 0.0, 0.0),
                        Vec3::new(0.0, 0.0, 20.0),
                        Vec3::new(0.0, 0.0, 0.0),
                    ]
                    .into_iter()
                    .map(|p| point + p)
                    .collect(),
                )
                .with_fill(Vec3::new(1.0, 0.0, 0.0)),
            ));
        }

        false
    }
}
