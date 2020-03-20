use crate::frame_parser::ActorHandler;
use crate::frame_parser::models::{ParsedFrameData, RumbleItemEvent};
use crate::network::frame_parser::FrameState;
use crate::frame_parser::utils::get_cars_player_actor_id;
use crate::Attribute;

pub struct RumbleItemHandler {
    pub item_name: String,
}

impl ActorHandler for RumbleItemHandler {
    fn create(&self, _data: &mut ParsedFrameData, _state: &mut FrameState, _actor_id: i32) {}

    fn update(&self, data: &mut ParsedFrameData, state: &mut FrameState, actor_id: i32,
              updated_attr: &String, _objects: &Vec<String>) {
        let attributes = try_opt!(state.actors.get(&actor_id));
        let player_actor_id = try_opt!(get_cars_player_actor_id(&attributes, state));
        let player_data = try_opt!(data.player_data.get_mut(&player_actor_id));

        match updated_attr.as_ref() {
            "TAGame.CarComponent_TA:Vehicle" => {
                player_data.power_up_active[state.frame] = Some(false);
                player_data.power_up[state.frame] = Some(self.item_name.clone());

                // Rumble item get event
                if player_data.power_up_active[state.frame - 1].is_none() {
                    if player_data.rumble_item_events.last()
                        .map(|event| event.demoed && self.item_name == event.item_name)
                        .unwrap_or(false) {
                        // the user was last demoed, do not create a new event
                        player_data.rumble_item_events.last_mut()
                            .map(|mut event| event.demoed = false);
                    } else {
                        player_data.rumble_item_events.push(RumbleItemEvent {
                            item_name: self.item_name.clone(),
                            frame_get: state.frame,
                            frame_use: None,
                            demoed: false,
                        })
                    }
                }
            }
            "TAGame.CarComponent_TA:ReplicatedActive" => {
                if let Some(Attribute::Byte(b)) = attributes.get("TAGame.CarComponent_TA:ReplicatedActive") {
                    let active = b % 2 == 1;
                    player_data.power_up[state.frame] = Some(self.item_name.clone());
                    player_data.power_up_active[state.frame] = Some(active.clone());

                    if !state.should_collect_stats() || state.frame == 0 {
                        return;
                    }

                    if !player_data.power_up_active[state.frame - 1].unwrap_or(false) && active {
                        // Rumble item use event
                        player_data.rumble_item_events.last_mut().map(|mut event|
                            event.frame_use = Some(state.frame));
                    }
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
                // When a spiked ball is frozen, there is not 'ball_freeze,True' row,
                // it just gets deleted immediately
                // Could also happen when the freeze is immediately broken
                // in theory this should not happen with other power ups?
                if state.should_collect_stats() {
                    if self.item_name == "BallFreeze" &&
                        !player_data.power_up_active[state.frame - 1].unwrap_or(true) {
                        player_data.rumble_item_events.last_mut()
                            .map(|mut event| event.frame_use = Some(state.frame));
                    }
                } else {
                    player_data.rumble_item_events.last_mut().map(|mut event| event.demoed = false);
                }

                player_data.power_up[state.frame] = None;
                player_data.power_up_active[state.frame] = None;
            }
        }
    }
}
