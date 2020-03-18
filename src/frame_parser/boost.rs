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
        let attributes = match state.actors.get(&actor_id) {
            Some(attributes) => attributes,
            _ => return,
        };

        let player_actor_id = match get_cars_player_actor_id(&attributes, state) {
            Some(id) => id,
            _ => return
        };

        let player_data = match data.player_data.get_mut(&player_actor_id) {
            Some(player_data) => player_data,
            _ => return,
        };

        match updated_attr.as_ref() {
            "TAGame.CarComponent_TA:ReplicatedActive" => {
                match attributes.get("TAGame.CarComponent_TA:ReplicatedActive") {
                    Some(Attribute::Byte(b)) => {
                        if b % 2 == 1 && player_data.boost[state.frame - 1].is_some() {
                            player_data.boost[state.frame] =
                                Some((player_data.boost[state.frame - 1].unwrap() - state.delta * BOOST_PER_SECOND).max(0.0))
                        }
                        player_data.boost_active[state.frame] = b.clone();
                    }
                    _ => return,
                }
            }
            "TAGame.CarComponent_Boost_TA:ReplicatedBoostAmount" => {
                match attributes.get("TAGame.CarComponent_Boost_TA:ReplicatedBoostAmount") {
                    Some(Attribute::Byte(b)) => {
                        player_data.boost[state.frame] = Some(b.clone() as f32);
                    }
                    _ => return,
                }
            }
            _ => return,
        }
    }

    fn destroy(&self, _data: &mut ParsedFrameData, _state: &mut FrameState, _actor_id: i32) {
    }
}

pub struct BoostPickupHandler {}

impl ActorHandler for BoostPickupHandler {
    fn create(&self, _data: &mut ParsedFrameData, _state: &mut FrameState, _actor_id: i32) {}

    fn update(&self, data: &mut ParsedFrameData, state: &mut FrameState, actor_id: i32,
              updated_attr: &String, _objects: &Vec<String>) {
        let attributes = match state.actors.get(&actor_id) {
            Some(attributes) => attributes,
            _ => return,
        };

        let car_actor_id = match updated_attr.as_ref() {
            "TAGame.VehiclePickup_TA:ReplicatedPickupData" => {
                match attributes.get("TAGame.VehiclePickup_TA:ReplicatedPickupData") {
                    Some(Attribute::Pickup(pickup)) => match pickup.instigator {
                        Some(actor) => actor.0,
                        _ => return
                    }
                    _ => return,
                }
            }
            "TAGame.VehiclePickup_TA:NewReplicatedPickupData" => {
                match attributes.get("TAGame.VehiclePickup_TA:NewReplicatedPickupData") {
                    Some(Attribute::PickupNew(pickup)) => match pickup.instigator {
                        Some(actor) => actor.0,
                        _ => return
                    }
                    _ => return,
                }
            }
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

    fn destroy(&self, data: &mut ParsedFrameData, state: &mut FrameState, actor_id: i32) {
        let attributes = match state.actors.get(&actor_id) {
            Some(attributes) => attributes,
            _ => return,
        };

        let car_actor_id = match attributes.get("TAGame.CarComponent_TA:Vehicle") {
            Some(Attribute::ActiveActor(actor)) => actor.actor.0,
            _ => return,
        };

        if car_actor_id == -1 {
            return;
        }

        let player_actor_id = match state.car_player_map.get(&car_actor_id) {
            Some(id) => id,
            _ => return
        };

        let player_data = match data.player_data.get_mut(&player_actor_id) {
            Some(player_data) => player_data,
            _ => return,
        };

        player_data.boost_active[state.frame] = 0;
        player_data.boost[state.frame] = None;
        player_data.boost_collect[state.frame] = false;
    }
}
