use crate::frame_parser::ActorHandler;
use crate::frame_parser::models::ParsedFrameData;
use crate::network::frame_parser::FrameState;
use crate::Attribute;
use crate::frame_parser::utils::get_cars_player_actor_id;

pub struct JumpHandler {}

impl ActorHandler for JumpHandler {
    fn create(&self, _data: &mut ParsedFrameData, _state: &mut FrameState, _actor_id: i32) {}

    fn update(&self, data: &mut ParsedFrameData, state: &mut FrameState, actor_id: i32,
              updated_attr: &String, _objects: &Vec<String>) {
        let attributes = try_opt!(state.actors.get(&actor_id));
        let player_actor_id = try_opt!(get_cars_player_actor_id(&attributes, state));
        let player_data = try_opt!(data.player_data.get_mut(&player_actor_id));
        match updated_attr.as_ref() {
            "TAGame.CarComponent_TA:ReplicatedActive" => {
                if let Some(Attribute::Byte(b)) = attributes.get("TAGame.CarComponent_TA:ReplicatedActive") {
                    player_data.jump_active[state.frame] = b.clone()
                }
            }
            _ => {}
        }
    }

    fn destroy(&self, data: &mut ParsedFrameData, state: &mut FrameState, actor_id: i32) {
        if_chain! {
            if let Some(attributes) = state.actors.get(&actor_id);
            if let Some(Attribute::ActiveActor(actor)) = attributes.get("TAGame.CarComponent_TA:Vehicle");
            if actor.actor.0 != -1;
            if let Some(player_actor_id) = state.car_player_map.get(&actor.actor.0);
            if let Some(player_data) = data.player_data.get_mut(&player_actor_id);
            then {
                player_data.jump_active[state.frame] = 0;
            }
        }
    }
}


pub struct DoubleJumpHandler {}

impl ActorHandler for DoubleJumpHandler {
    fn create(&self, _data: &mut ParsedFrameData, _state: &mut FrameState, _actor_id: i32) {}

    fn update(&self, data: &mut ParsedFrameData, state: &mut FrameState, actor_id: i32,
              updated_attr: &String, _objects: &Vec<String>) {
        let attributes = try_opt!(state.actors.get(&actor_id));
        let player_actor_id = try_opt!(get_cars_player_actor_id(&attributes, state));
        let player_data = try_opt!(data.player_data.get_mut(&player_actor_id));
        match updated_attr.as_ref() {
            "TAGame.CarComponent_TA:ReplicatedActive" => {
                if let Some(Attribute::Byte(b)) = attributes.get("TAGame.CarComponent_TA:ReplicatedActive") {
                    player_data.double_jump_active[state.frame] = b.clone()
                }
            }
            _ => {}
        }
    }

    fn destroy(&self, data: &mut ParsedFrameData, state: &mut FrameState, actor_id: i32) {
        if_chain! {
            if let Some(attributes) = state.actors.get(&actor_id);
            if let Some(Attribute::ActiveActor(actor)) = attributes.get("TAGame.CarComponent_TA:Vehicle");
            if actor.actor.0 != -1;
            if let Some(player_actor_id) = state.car_player_map.get(&actor.actor.0);
            if let Some(player_data) = data.player_data.get_mut(&player_actor_id);
            then {
                player_data.double_jump_active[state.frame] = 0;
            }
        }
    }
}


pub struct DodgeHandler {}

impl ActorHandler for DodgeHandler {
    fn create(&self, _data: &mut ParsedFrameData, _state: &mut FrameState, _actor_id: i32) {}

    fn update(&self, data: &mut ParsedFrameData, state: &mut FrameState, actor_id: i32,
              updated_attr: &String, _objects: &Vec<String>) {
        let attributes = try_opt!(state.actors.get(&actor_id));
        let player_actor_id = try_opt!(get_cars_player_actor_id(&attributes, state));
        let player_data = try_opt!(data.player_data.get_mut(&player_actor_id));
        match updated_attr.as_ref() {
            "TAGame.CarComponent_TA:ReplicatedActive" => {
                if let Some(Attribute::Byte(b)) = attributes.get("TAGame.CarComponent_TA:ReplicatedActive") {
                    player_data.dodge_active[state.frame] = b.clone()
                }
            }
            _ => return,
        }
    }

    fn destroy(&self, data: &mut ParsedFrameData, state: &mut FrameState, actor_id: i32) {
        if_chain! {
            if let Some(attributes) = state.actors.get(&actor_id);
            if let Some(Attribute::ActiveActor(actor)) = attributes.get("TAGame.CarComponent_TA:Vehicle");
            if actor.actor.0 != -1;
            if let Some(player_actor_id) = state.car_player_map.get(&actor.actor.0);
            if let Some(player_data) = data.player_data.get_mut(&player_actor_id);
            then {
                player_data.dodge_active[state.frame] = 0;
            }
        }
    }
}
