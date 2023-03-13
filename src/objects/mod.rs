mod building;
mod highway;
mod park;
mod railway;

use crate::renderer::{Point, Renderable};

pub use building::Building;
pub use highway::Highway;
pub use park::Park;
pub use railway::Railway;

pub trait Object {
    fn get_renderables(&self, points: &[Point]) -> Vec<Renderable>;
}
