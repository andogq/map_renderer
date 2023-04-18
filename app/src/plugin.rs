use std::{cell::RefCell, rc::Rc};

use renderer::{window::WindowEvent, RenderStep};

use crate::map_data::MapData;

pub(crate) trait Plugin<A> {
    /// Called to attach map data to plugin.
    fn with_map_data(&mut self, map_data: Rc<MapData>);

    /// Called upon initialisation to create render step.
    fn get_render_step(&self) -> Rc<RefCell<dyn RenderStep>>;

    /// Called every time an event on the window occurs, returning true or false depending on
    /// whether the plugin state changed (and a re-render is requried).
    fn handle_event(&mut self, app_state: A, event: WindowEvent) -> bool;
}
