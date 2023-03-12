mod objects;
mod renderable;

use objects::{Building, Highway, Object, Park, Railway};
use osmpbf::{Element, ElementReader, TagIter};
use piet::{kurbo::PathEl, Color, RenderContext};
use piet_common::Device;
use renderable::{Point};
use std::{collections::HashMap, f64::consts::PI, mem};

const SIZE: usize = 500;
const SCALE: usize = 8;

#[derive(Debug)]
struct Bounding {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

#[derive(Debug)]
enum RelationMemberType {
    Node,
    Way,
    Relation,
}
impl From<osmpbf::RelMemberType> for RelationMemberType {
    fn from(member_type: osmpbf::RelMemberType) -> Self {
        use RelationMemberType::*;

        match member_type {
            osmpbf::RelMemberType::Node => Node,
            osmpbf::RelMemberType::Way => Way,
            osmpbf::RelMemberType::Relation => Relation,
        }
    }
}

#[derive(Debug)]
struct RelationMember {
    pub role: Option<String>,
    pub id: i64,
    pub member_type: RelationMemberType,
}

#[derive(Debug)]
struct Relation {
    pub tags: HashMap<String, String>,
    pub members: Vec<RelationMember>,
}

struct Way {
    pub tags: HashMap<String, String>,
    pub nodes: Vec<i64>,
}

impl Way {
    pub fn get_way_type(&self) -> Option<Box<dyn Object>> {
        if self.tags.contains_key("highway") {
            return Some(Box::new(Highway::from_tags(&self.tags).unwrap()));
        } else if self
            .tags
            .get("leisure")
            .filter(|&ty| ty == "park")
            .is_some()
        {
            return Some(Box::new(Park));
        } else if self.tags.contains_key("railway") {
            return Some(Box::new(Railway));
        } else if self.tags.contains_key("building") {
            return Some(Box::new(Building));
        }

        None
    }
}

struct Node {
    pub x: f64,
    pub y: f64,
    pub tags: HashMap<String, String>,
}

impl Node {
    pub fn new(x: f64, y: f64) -> Node {
        Node {
            x,
            y,
            tags: HashMap::new(),
        }
    }

    pub fn from_lon_lat(lon: f64, lat: f64) -> Node {
        let x = lon;
        let y = f64::ln(f64::tan((PI / 4.0) + (lat / 2.0)));
        Node {
            x,
            y,
            tags: HashMap::new(),
        }
    }
}

impl From<Node> for piet::kurbo::Point {
    fn from(value: Node) -> Self {
        piet::kurbo::Point {
            x: value.x,
            y: value.y,
        }
    }
}

fn convert_tags(iter: TagIter) -> HashMap<String, String> {
    iter.map(|(key, value)| (key.to_string(), value.to_string()))
        .collect()
}

fn main() -> osmpbf::Result<()> {
    let reader = ElementReader::from_path("data/cbd.osm.pbf").expect("data/cbd.osm.pbf to exist");

    let mut nodes: HashMap<i64, Node> = HashMap::new();
    let mut ways: HashMap<i64, Way> = HashMap::new();
    let mut relations: HashMap<i64, Relation> = HashMap::new();

    let mut bounding: Option<Bounding> = None;

    reader.for_each(|element| {
        match element {
            Element::Node(_) | Element::DenseNode(_) => {
                let (id, node) = match element {
                    Element::Node(node) => (node.id(), Node::from_lon_lat(node.lon(), node.lat())),
                    Element::DenseNode(node) => {
                        (node.id(), Node::from_lon_lat(node.lon(), node.lat()))
                    }
                    _ => unreachable!("can only match to node or dense node"),
                };

                // Check the node bounding
                bounding = Some(if let Some(bounding) = &bounding {
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
                });

                nodes.insert(id, node);
            }
            Element::Way(way) => {
                ways.insert(
                    way.id(),
                    Way {
                        tags: convert_tags(way.tags()),
                        nodes: way.refs().collect(),
                    },
                );
            }
            Element::Relation(relation) => {
                relations.insert(
                    relation.id(),
                    Relation {
                        tags: convert_tags(relation.tags()),
                        members: relation
                            .members()
                            .map(|member| RelationMember {
                                role: member.role().ok().map(|role| role.to_string()),
                                id: member.member_id,
                                member_type: member.member_type.into(),
                            })
                            .collect(),
                    },
                );
            }
        };
    })?;

    let bounding = bounding.unwrap();
    let d_lat = bounding.max_y - bounding.min_y;
    let d_lon = bounding.max_x - bounding.min_x;
    let scaling = f64::max(d_lat, d_lon);

    let mut device = Device::new().unwrap();
    let mut bitmap = device
        .bitmap_target(SIZE * SCALE, SIZE * SCALE, SCALE as f64)
        .unwrap();
    let mut ctx = bitmap.render_context();

    ctx.clear(None, Color::WHITE);

    for (_way_id, way) in ways.iter() {
        if let Some(way_type) = way.get_way_type() {
            let points = way
                .nodes
                .iter()
                .filter_map(|node_id| nodes.get(node_id))
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
