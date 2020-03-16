pub mod models;
mod game_event;

use crate::network::frame_parser::FrameState;
use crate::frame_parser::models::ParsedFrameData;
use crate::frame_parser::game_event::GameEventHandler;

pub trait ActorHandler {
    fn update(&self, data: &mut ParsedFrameData, state: &mut FrameState,
              actor_id: i32, updated_attr: &String,
              objects: &Vec<String>);
}

pub fn get_handler(object_name: &String) -> Option<Box<dyn ActorHandler>> {
    if object_name.starts_with("Archetypes.GameEvent.GameEvent_") {
        return Some(Box::new(GameEventHandler {}));
    }
    match object_name.as_ref() {
        "" => None,
        _ => None,
    }
}
