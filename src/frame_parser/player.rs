use crate::frame_parser::ActorHandler;
use crate::frame_parser::models::ParsedFrameData;
use crate::network::frame_parser::FrameState;
use crate::frame_parser::models::PlayerData;
use crate::Attribute;
use crate::frame_parser::utils::get_remote_id;
use std::collections::HashSet;

pub struct PlayerHandler {}

impl ActorHandler for PlayerHandler {
    fn create(&self, data: &mut ParsedFrameData, state: &mut FrameState, actor_id: i32) {
        if data.player_data.contains_key(&actor_id) {
            return;
        }
        let mut player_data = PlayerData::with_capacity(state.total_frames);
        player_data.new_frame();
        data.player_data.insert(actor_id, player_data);
    }

    fn update(&self, data: &mut ParsedFrameData, state: &mut FrameState, actor_id: i32,
              updated_attr: &String, _: &Vec<String>) {
        let attributes = match state.actors.get(&actor_id) {
            None => return,
            Some(attributes) => attributes
        };

        let player_data = data.player_data.get_mut(&actor_id).unwrap();

        match updated_attr.as_ref() {
            "Engine.PlayerReplicationInfo:Team" => {
                match attributes.get("Engine.PlayerReplicationInfo:Team") {
                    Some(Attribute::ActiveActor(actor)) => {
                        if actor.actor.0 != -1 {
                            player_data.team_actor = actor.actor.0;
                        }
                    }
                    _ => return,
                };
            }
            "Engine.PlayerReplicationInfo:PlayerName" => {
                match attributes.get("Engine.PlayerReplicationInfo:PlayerName") {
                    Some(Attribute::String(name)) => player_data.name = Some(name.clone()),
                    _ => return,
                };
            }
            "Engine.PlayerReplicationInfo:UniqueId" => {
                match get_remote_id(attributes, "Engine.PlayerReplicationInfo:UniqueId") {
                    Some(id) => player_data.remote_id = Some(id),
                    _ => return,
                };
            }
            "Engine.PlayerReplicationInfo:Ping" => {
                match attributes.get("Engine.PlayerReplicationInfo:Ping") {
                    Some(Attribute::Byte(ping)) => player_data.ping.push(Some(ping.clone())),
                    _ => return,
                };
            }
            "TAGame.CameraSettingsActor_TA:bUsingSecondaryCamera" => {
                match attributes.get("TAGame.CameraSettingsActor_TA:bUsingSecondaryCamera") {
                    Some(Attribute::Boolean(ball_cam)) => player_data.ball_cam.push(Some(ball_cam.clone())),
                    _ => return,
                };
            }
            "TAGame.PRI_TA:TimeTillItem" => {
                if player_data.time_till_power_up.is_none() {
                    player_data.time_till_power_up = Some(Vec::with_capacity(state.total_frames));
                }

                match attributes.get("TAGame.PRI_TA:TimeTillItem") {
                    Some(Attribute::Int(time)) => player_data.time_till_power_up.as_mut().unwrap().push(Some(time.clone())),
                    _ => return,
                };
            }
            "TAGame.PRI_TA:PartyLeader" => {
                let leader_id = match get_remote_id(attributes, "TAGame.PRI_TA:PartyLeader") {
                    Some(leader_id) => leader_id,
                    _ => return,
                };

                if !data.parties.contains_key(&leader_id) {
                    data.parties.insert(leader_id.clone(), HashSet::new());
                }

                match get_remote_id(attributes, "Engine.PlayerReplicationInfo:UniqueId") {
                    Some(id) => {
                        data.parties.get_mut(&leader_id).unwrap().insert(id);
                    },
                    _ => return,
                }
            }
            _ => return
        }
    }
}
