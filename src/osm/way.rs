use crate::objects::*;

use super::Tags;

pub struct Way {
    pub tags: Tags,
    pub nodes: Vec<i64>,
}

impl Way {
    pub fn get_way_type(&self) -> Option<Box<dyn Object>> {
        if self.tags.contains("highway") {
            return Some(Box::new(Highway::from_tags(&self.tags).unwrap()));
        } else if self
            .tags
            .get("leisure")
            .filter(|&ty| ty == "park")
            .is_some()
        {
            return Some(Box::new(Park));
        } else if self.tags.contains("railway") {
            return Some(Box::new(Railway));
        } else if self.tags.contains("building") {
            return Some(Box::new(Building));
        }

        None
    }
}

impl From<osmpbf::Way<'_>> for Way {
    fn from(way: osmpbf::Way) -> Self {
        Self {
            nodes: way.refs().collect(),
            tags: way.tags().into(),
        }
    }
}
