use crate::frame_parser::ActorHandler;
use crate::frame_parser::models::ParsedFrameData;
use crate::network::frame_parser::FrameState;
use crate::Attribute;

pub struct JumpHandler {}

impl ActorHandler for JumpHandler {
    fn create(&self, _data: &mut ParsedFrameData, _state: &mut FrameState, _actor_id: i32) {}

    fn update(&self, data: &mut ParsedFrameData, state: &mut FrameState, actor_id: i32,
              updated_attr: &String, _objects: &Vec<String>) {
        let attributes = match state.actors.get(&actor_id) {
            Some(attributes) => attributes,
            _ => return,
        };

        let car_actor_id = match attributes.get("TAGame.CarComponent_TA:Vehicle") {
            Some(Attribute::ActiveActor(actor)) => actor.actor.0,
            _ => return,
        };

        let player_actor_id = match state.actors.get(&car_actor_id) {
            Some(attributes) => match attributes.get("Engine.Pawn:PlayerReplicationInfo") {
                Some(Attribute::ActiveActor(actor)) => actor.actor.0,
                _ => return,
            },
            _ => return,
        };

        let player_data = match data.player_data.get_mut(&player_actor_id) {
            Some(player_data) => player_data,
            _ => return,
        };

        match updated_attr.as_ref() {
            "TAGame.CarComponent_TA:ReplicatedActive" => {
                match attributes.get("TAGame.CarComponent_TA:ReplicatedActive") {
                    Some(Attribute::Byte(b)) => player_data.jump_active[state.frame] = Some(b.clone()),
                    _ => return,
                }
            }
            _ => return,
        }
    }
}


pub struct DoubleJumpHandler {}

impl ActorHandler for DoubleJumpHandler {
    fn create(&self, _data: &mut ParsedFrameData, _state: &mut FrameState, _actor_id: i32) {}

    fn update(&self, data: &mut ParsedFrameData, state: &mut FrameState, actor_id: i32,
              updated_attr: &String, _objects: &Vec<String>) {
        let attributes = match state.actors.get(&actor_id) {
            Some(attributes) => attributes,
            _ => return,
        };

        let car_actor_id = match attributes.get("TAGame.CarComponent_TA:Vehicle") {
            Some(Attribute::ActiveActor(actor)) => actor.actor.0,
            _ => return,
        };

        let player_actor_id = match state.actors.get(&car_actor_id) {
            Some(attributes) => match attributes.get("Engine.Pawn:PlayerReplicationInfo") {
                Some(Attribute::ActiveActor(actor)) => actor.actor.0,
                _ => return,
            },
            _ => return,
        };

        let player_data = match data.player_data.get_mut(&player_actor_id) {
            Some(player_data) => player_data,
            _ => return,
        };

        match updated_attr.as_ref() {
            "TAGame.CarComponent_TA:ReplicatedActive" => {
                match attributes.get("TAGame.CarComponent_TA:ReplicatedActive") {
                    Some(Attribute::Byte(b)) => player_data.double_jump_active[state.frame] = Some(b.clone()),
                    _ => return,
                }
            }
            _ => return,
        }
    }
}


pub struct DodgeHandler {}

impl ActorHandler for DodgeHandler {
    fn create(&self, _data: &mut ParsedFrameData, _state: &mut FrameState, _actor_id: i32) {}

    fn update(&self, data: &mut ParsedFrameData, state: &mut FrameState, actor_id: i32,
              updated_attr: &String, _objects: &Vec<String>) {
        let attributes = match state.actors.get(&actor_id) {
            Some(attributes) => attributes,
            _ => return,
        };

        let car_actor_id = match attributes.get("TAGame.CarComponent_TA:Vehicle") {
            Some(Attribute::ActiveActor(actor)) => actor.actor.0,
            _ => return,
        };

        let player_actor_id = match state.actors.get(&car_actor_id) {
            Some(attributes) => match attributes.get("Engine.Pawn:PlayerReplicationInfo") {
                Some(Attribute::ActiveActor(actor)) => actor.actor.0,
                _ => return,
            },
            _ => return,
        };

        let player_data = match data.player_data.get_mut(&player_actor_id) {
            Some(player_data) => player_data,
            _ => return,
        };

        match updated_attr.as_ref() {
            "TAGame.CarComponent_TA:ReplicatedActive" => {
                match attributes.get("TAGame.CarComponent_TA:ReplicatedActive") {
                    Some(Attribute::Byte(b)) => player_data.dodge_active[state.frame] = Some(b.clone()),
                    _ => return,
                }
            }
            _ => return,
        }
    }
}
