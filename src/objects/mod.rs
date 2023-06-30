mod building;
mod highway;
mod park;
mod railway;

use crate::Point;

pub use building::Building;
pub use highway::Highway;
pub use park::Park;
pub use railway::Railway;
use renderer::render_steps::canvas::Path;

pub(crate) trait Object {
    fn get_paths(&self, points: &[Point]) -> Vec<Path>;
}
