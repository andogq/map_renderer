mod node;
mod relation;
mod tags;
mod way;

use std::{collections::HashMap, io::Read};

pub use node::Node;
use osmpbf::{Element, ElementReader};
pub use relation::*;
pub use tags::Tags;
pub use way::Way;

pub struct Osm {
    pub nodes: HashMap<i64, Node>,
    pub ways: HashMap<i64, Way>,
    pub relations: HashMap<i64, Relation>,
}

impl Osm {
    pub fn from_reader<R: Send + Read>(reader: ElementReader<R>) -> osmpbf::Result<Self> {
        let mut nodes: HashMap<i64, Node> = HashMap::new();
        let mut ways: HashMap<i64, Way> = HashMap::new();
        let mut relations: HashMap<i64, Relation> = HashMap::new();

        reader.for_each(|element| {
            match element {
                Element::Node(_) | Element::DenseNode(_) => {
                    let (id, node): (i64, Node) = match element {
                        Element::Node(node) => (node.id(), node.into()),
                        Element::DenseNode(node) => (node.id(), node.into()),
                        _ => unreachable!("can only match to node or dense node"),
                    };

                    nodes.insert(id, node);
                }
                Element::Way(way) => {
                    ways.insert(way.id(), way.into());
                }
                Element::Relation(relation) => {
                    relations.insert(relation.id(), relation.into());
                }
            };
        })?;

        Ok(Osm {
            nodes,
            ways,
            relations,
        })
    }
}
