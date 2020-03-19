use crate::frame_parser::ActorHandler;
use crate::frame_parser::models::ParsedFrameData;
use crate::network::frame_parser::FrameState;
use crate::frame_parser::utils::get_cars_player_actor_id;
use crate::Attribute;

pub struct RumbleItemHandler {}

impl ActorHandler for RumbleItemHandler {
    fn create(&self, _data: &mut ParsedFrameData, _state: &mut FrameState, _actor_id: i32) {
    }

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
            "TAGame.CarComponent_TA:Vehicle" => {
                player_data.power_up_active[state.frame] = Some(false);
                let item_name = match state.actor_objects.get(&actor_id)
                    .map(|x| x.replace("Archetypes.SpecialPickups.SpecialPickup_", "")) {
                    Some(item_name) => item_name,
                    _ => return,
                };
                player_data.power_up[state.frame] = Some(item_name);
            }
            "TAGame.CarComponent_TA:ReplicatedActive" => {
                match attributes.get("TAGame.CarComponent_TA:ReplicatedActive") {
                    Some(Attribute::Byte(b)) => {
                        let item_name = match state.actor_objects.get(&actor_id)
                            .map(|x| x.replace("Archetypes.SpecialPickups.SpecialPickup_", "")) {
                            Some(item_name) => item_name,
                            _ => return,
                        };
                        player_data.power_up[state.frame] = Some(item_name);
                        player_data.power_up_active[state.frame] = Some(b % 2 == 1);
                    },
                    _ => return,
                }
            }
            _ => return,
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

        player_data.power_up[state.frame] = None;
        player_data.power_up_active[state.frame] = None;
    }
}
