use crate::frame_parser::ActorHandler;
use crate::frame_parser::models::ParsedFrameData;
use crate::network::frame_parser::FrameState;
use crate::frame_parser::utils::get_cars_player_actor_id;
use crate::Attribute;

pub const BOOST_PER_SECOND: f32 = 80.0 / 0.93;

pub struct BoostHandler {}

impl ActorHandler for BoostHandler {
    fn create(&self, _data: &mut ParsedFrameData, _state: &mut FrameState, _actor_id: i32) {}

    fn update(&self, data: &mut ParsedFrameData, state: &mut FrameState, actor_id: i32,
              updated_attr: &String, _objects: &Vec<String>) {
        let attributes = try_opt!(state.actors.get(&actor_id));
        let player_actor_id = try_opt!(get_cars_player_actor_id(&attributes, state));
        let player_data = try_opt!(data.player_data.get_mut(&player_actor_id));
        match updated_attr.as_ref() {
            "TAGame.CarComponent_TA:ReplicatedActive" => {
                if let Some(Attribute::Byte(b)) = attributes.get("TAGame.CarComponent_TA:ReplicatedActive") {
                    let active = b % 2 == 1;
                    if active && player_data.boost[state.frame - 1].is_some() {
                        player_data.boost[state.frame] =
                            Some((player_data.boost[state.frame - 1].unwrap() - state.delta * BOOST_PER_SECOND).max(0.0))
                    }
                    player_data.boost_active[state.frame] = active;
                }
            }
            "TAGame.CarComponent_Boost_TA:ReplicatedBoostAmount" => {
                if let Some(Attribute::Byte(b)) = attributes.get("TAGame.CarComponent_Boost_TA:ReplicatedBoostAmount") {
                    player_data.boost[state.frame] = Some(b.clone() as f32);
                }
            }
            _ => return,
        }
    }

    fn destroy(&self, data: &mut ParsedFrameData, state: &mut FrameState, actor_id: i32) {
        if let Some(attributes) = state.actors.get(&actor_id) {
            if let Some(Attribute::ActiveActor(actor)) = attributes.get("TAGame.CarComponent_TA:Vehicle") {
                if actor.actor.0 == -1 {
                    return;
                }

                let player_actor_id = try_opt!(state.car_player_map.get(&actor.actor.0));
                let player_data = try_opt!(data.player_data.get_mut(&player_actor_id));
                player_data.boost_active[state.frame] = false;
            }
        }
    }
}

pub struct BoostPickupHandler {}

impl ActorHandler for BoostPickupHandler {
    fn create(&self, _data: &mut ParsedFrameData, _state: &mut FrameState, _actor_id: i32) {}

    fn update(&self, data: &mut ParsedFrameData, state: &mut FrameState, actor_id: i32,
              updated_attr: &String, _objects: &Vec<String>) {
        if let Some(attributes) = state.actors.get(&actor_id) {
            let car_actor_id = match updated_attr.as_ref() {
                "TAGame.VehiclePickup_TA:ReplicatedPickupData" => {
                    match attributes.get("TAGame.VehiclePickup_TA:ReplicatedPickupData") {
                        Some(Attribute::Pickup(pickup)) => try_opt!(pickup.instigator).0,
                        _ => return,
                    }
                }
                "TAGame.VehiclePickup_TA:NewReplicatedPickupData" => {
                    match attributes.get("TAGame.VehiclePickup_TA:NewReplicatedPickupData") {
                        Some(Attribute::PickupNew(pickup)) => try_opt!(pickup.instigator).0,
                        _ => return,
                    }
                }
                _ => return,
            };

            if let Some(Attribute::ActiveActor(actor)) = try_opt!(state.actors.get(&car_actor_id)).get("Engine.Pawn:PlayerReplicationInfo") {
                let player_data = try_opt!(data.player_data.get_mut(&actor.actor.0));

                let current_boost = player_data.boost[state.frame];

                let mut prev_boost: Option<f32> = None;
                for i in state.frame - 1..0 {
                    if player_data.boost[i].is_some() {
                        prev_boost = player_data.boost[i].clone();
                        break;
                    }
                }

                // Ignore any phantom boosts
                if prev_boost.is_some() && current_boost.is_some() && prev_boost.unwrap() < 255.0 && current_boost.unwrap() > prev_boost.unwrap() {
                    player_data.boost_collect[state.frame] = true;
                }
            }
        }
    }

    fn destroy(&self, data: &mut ParsedFrameData, state: &mut FrameState, actor_id: i32) {
        if let Some(attributes) = state.actors.get(&actor_id) {
            if let Some(Attribute::ActiveActor(actor)) = attributes.get("TAGame.CarComponent_TA:Vehicle") {
                if actor.actor.0 == -1 {
                    return;
                }

                let player_actor_id = try_opt!(state.car_player_map.get(&actor.actor.0));
                let player_data = try_opt!(data.player_data.get_mut(&player_actor_id));
                player_data.boost_active[state.frame] = false;
                player_data.boost[state.frame] = None;
                player_data.boost_collect[state.frame] = false;
            }
        }
    }
}
