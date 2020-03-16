mod ball;
pub mod models;
mod game_event;

use crate::network::frame_parser::FrameState;
use crate::frame_parser::models::{ParsedFrameData, BallType};
use crate::frame_parser::game_event::GameEventHandler;
use crate::frame_parser::ball::BallHandler;

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
        "Archetypes.Ball.Ball_Default" => Some(Box::new(BallHandler { ball_type: BallType::Default })),
        "Archetypes.Ball.Ball_Basketball" => Some(Box::new(BallHandler { ball_type: BallType::Basketball })),
        "Archetypes.Ball.Ball_BasketBall" => Some(Box::new(BallHandler { ball_type: BallType::Basketball })),
        "Archetypes.Ball.Ball_Puck" => Some(Box::new(BallHandler { ball_type: BallType::Puck })),
        "Archetypes.Ball.CubeBall" => Some(Box::new(BallHandler { ball_type: BallType::Cube })),
        "Archetypes.Ball.Ball_Breakout" => Some(Box::new(BallHandler { ball_type: BallType::Breakout })),
        _ => None,
    }
}
