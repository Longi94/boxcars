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
        data.player_data.insert(actor_id, PlayerData::with_capacity(state.total_frames));
    }

    fn update(&self, data: &mut ParsedFrameData, state: &mut FrameState, actor_id: i32,
              updated_attr: &String, _: &Vec<String>) {
        let attributes = match state.actors.get(&actor_id) {
            Some(attributes) => attributes,
            _ => return,
        };

        let player_data = match data.player_data.get_mut(&actor_id) {
            Some(player_data) => player_data,
            _ => return,
        };

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
                    Some(Attribute::Byte(ping)) => player_data.ping[state.frame] = Some(ping.clone()),
                    _ => return,
                };
            }
            "TAGame.CameraSettingsActor_TA:bUsingSecondaryCamera" => {
                match attributes.get("TAGame.CameraSettingsActor_TA:bUsingSecondaryCamera") {
                    Some(Attribute::Boolean(ball_cam)) => player_data.ball_cam[state.frame] = Some(ball_cam.clone()),
                    _ => return,
                };
            }
            "TAGame.PRI_TA:TimeTillItem" => {
                if player_data.time_till_power_up.is_none() {
                    let mut vec: Vec<Option<i32>> = Vec::with_capacity(state.total_frames);
                    vec.resize(state.total_frames, None);
                    player_data.time_till_power_up = Some(vec);
                }

                match attributes.get("TAGame.PRI_TA:TimeTillItem") {
                    Some(Attribute::Int(time)) => player_data.time_till_power_up.as_mut()
                        .unwrap()[state.frame] = Some(time.clone()),
                    _ => return,
                };
            }
            "TAGame.PRI_TA:PartyLeader" => {
                let leader_id = match get_remote_id(attributes, "TAGame.PRI_TA:PartyLeader") {
                    Some(leader_id) => leader_id,
                    _ => return,
                };

                player_data.party_leader = Some(leader_id.clone());

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
            "TAGame.PRI_TA:MatchScore" => {
                match attributes.get("TAGame.PRI_TA:MatchScore") {
                    Some(Attribute::Int(score)) => player_data.match_score = Some(score.clone()),
                    _ => return,
                }
            }
            "TAGame.PRI_TA:Title" => {
                match attributes.get("TAGame.PRI_TA:Title") {
                    Some(Attribute::Int(title)) => player_data.title = Some(title.clone()),
                    _ => return,
                }
            }
            "TAGame.PRI_TA:TotalXP" => {
                match attributes.get("TAGame.PRI_TA:TotalXP") {
                    Some(Attribute::Int(total_xp)) => player_data.total_xp = Some(total_xp.clone()),
                    _ => return,
                }
            }
            "TAGame.PRI_TA:SteeringSensitivity" => {
                match attributes.get("TAGame.PRI_TA:SteeringSensitivity") {
                    Some(Attribute::Float(sens)) => player_data.steering_sensitivity = Some(sens.clone()),
                    _ => return,
                }
            }
            _ => return
        }
    }
}
