use std::collections::{HashMap, HashSet};

use crate::{osm::Osm, Bounding};
use glam::Vec3;
use renderer::{
    ogl::{DrawType, Program, VertexData, VertexFormat, VertexType},
    RenderStep,
};

pub(crate) struct Network {
    intersections: HashMap<i64, HashSet<i64>>,
}

impl Network {
    pub fn new(data: &Osm) -> Self {
        let route_ways = data
            .ways
            .iter()
            .filter(|(_, way)| {
                way.tags
                    .get("highway")
                    .map(|value| {
                        [
                            "trunk",
                            "primary",
                            "secondary",
                            "tertiary",
                            "unclassified",
                            "motorway",
                            "residential",
                            "service",
                            "road",
                            "living_street",
                            "track",
                        ]
                        .contains(&value.as_str())
                    })
                    .unwrap_or_default()
            })
            .collect::<HashMap<_, _>>();

        let intersections = {
            let mut intersections =
                route_ways
                    .iter()
                    .fold(HashMap::new(), |mut hash_map, (&&way_id, way)| {
                        way.nodes.iter().for_each(|&node_id| {
                            hash_map
                                .entry(node_id)
                                .or_insert_with(HashSet::new)
                                .insert(way_id);
                        });

                        hash_map
                    });

            intersections.retain(|_, ways| ways.len() > 1);

            intersections
        };

        dbg!(intersections.len());

        Network { intersections }
    }
}

pub(crate) struct PathFinder {
    start: Vec3,
    points: Vec<Vec3>,
}

impl PathFinder {
    pub fn new(osm_data: &Osm, bounding: &Bounding) -> Self {
        let network = Network::new(osm_data);

        let d_lat = bounding.max_y - bounding.min_y;
        let d_lon = bounding.max_x - bounding.min_x;
        let scaling = 500_f64 / f64::max(d_lat, d_lon);

        // Select a random start node
        let start = {
            let node = osm_data.nodes.values().next().unwrap();

            let x = (node.x - bounding.min_x - (d_lat / 2.0)) * scaling;
            let y = (node.y - bounding.min_y - (d_lon / 2.0)) * scaling;

            Vec3::new(x as f32, 0.0, y as f32)
        };

        Self {
            start,
            points: network
                .intersections
                .keys()
                .map(|node_id| {
                    let node = osm_data.nodes.get(node_id).unwrap();

                    let x = (node.x - bounding.min_x - (d_lat / 2.0)) * scaling;
                    let y = (node.y - bounding.min_y - (d_lon / 2.0)) * scaling;

                    Vec3::new(x as f32, 0.0, y as f32)
                })
                .collect(),
        }
    }
}

impl RenderStep for PathFinder {
    fn build_programs(
        &self,
        gl: &mut renderer::ogl::OpenGl,
    ) -> Vec<std::rc::Rc<std::cell::RefCell<renderer::ogl::Program>>> {
        vec![gl
            .add_program(
                Program::from_directory("app/src/shaders/path_finder")
                    .unwrap()
                    .with_format(&[VertexFormat::new(3, VertexType::Float)])
                    .with_draw_type(DrawType::Points),
            )
            .unwrap()]
    }

    fn get_vertices(&self) -> Vec<Vec<u8>> {
        vec![self.points.iter().flat_map(|p| p.get_bytes()).collect()]
    }
}
