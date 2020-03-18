use crate::frame_parser::ActorHandler;
use crate::frame_parser::models::ParsedFrameData;
use crate::network::frame_parser::FrameState;
use crate::frame_parser::models::PlayerData;
use crate::{Attribute, Product};
use crate::frame_parser::utils::get_remote_id;
use crate::attributes::ProductValue;
use std::collections::HashSet;
use std::ops::Deref;

pub struct PlayerHandler {}

impl ActorHandler for PlayerHandler {
    fn create(&self, data: &mut ParsedFrameData, state: &mut FrameState, actor_id: i32) {
        if data.player_data.contains_key(&actor_id) {
            return;
        }
        data.player_data.insert(actor_id, PlayerData::with_capacity(state.total_frames));
    }

    fn update(&self, data: &mut ParsedFrameData, state: &mut FrameState, actor_id: i32,
              updated_attr: &String, objects: &Vec<String>) {
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
                    }
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
            "TAGame.PRI_TA:ClientLoadout" => {
                match attributes.get("TAGame.PRI_TA:ClientLoadout") {
                    Some(Attribute::Loadout(loadout)) => player_data.loadout = loadout.deref().clone(),
                    _ => return,
                }
            }
            "TAGame.PRI_TA:ClientLoadouts" => {
                match attributes.get("TAGame.PRI_TA:ClientLoadouts") {
                    Some(Attribute::TeamLoadout(loadouts)) => {
                        let team_actor_id = match attributes.get("Engine.PlayerReplicationInfo:Team") {
                            Some(Attribute::ActiveActor(actor)) => actor.actor.0,
                            _ => return,
                        };

                        let is_orange = match state.actor_objects.get(&team_actor_id) {
                            Some(object_name) => object_name.ends_with("1"),
                            _ => return,
                        };

                        if is_orange {
                            player_data.loadout = loadouts.orange.clone();
                        } else {
                            player_data.loadout = loadouts.blue.clone();
                        }
                    }
                    _ => return,
                }
            }
            "TAGame.PRI_TA:ClientLoadoutOnline" => {
                match attributes.get("TAGame.PRI_TA:ClientLoadoutOnline") {
                    Some(Attribute::LoadoutOnline(paints)) => {
                        player_data.loadout_paints.body = get_paint_value(&paints[0], objects);
                        player_data.loadout_paints.decal = get_paint_value(&paints[1], objects);
                        player_data.loadout_paints.wheels = get_paint_value(&paints[2], objects);
                        player_data.loadout_paints.boost = get_paint_value(&paints[3], objects);
                        player_data.loadout_paints.antenna = get_paint_value(&paints[4], objects);
                        player_data.loadout_paints.topper = get_paint_value(&paints[5], objects);
                        player_data.loadout_paints.trail = get_paint_value(&paints[14], objects);
                        player_data.loadout_paints.goal_explosion = get_paint_value(&paints[15], objects);
                        player_data.loadout_paints.banner = get_paint_value(&paints[16], objects);
                        player_data.loadout_paints.avatar_border = get_paint_value(&paints[20], objects);
                    }
                    _ => return,
                }
            }
            "TAGame.PRI_TA:ClientLoadoutsOnline" => {
                match attributes.get("TAGame.PRI_TA:ClientLoadoutsOnline") {
                    Some(Attribute::LoadoutsOnline(team_paints)) => {
                        let team_actor_id = match attributes.get("Engine.PlayerReplicationInfo:Team") {
                            Some(Attribute::ActiveActor(actor)) => actor.actor.0,
                            _ => return,
                        };

                        let paints = match state.actor_objects.get(&team_actor_id) {
                            Some(object_name) => {
                                let is_orange = object_name.ends_with("1");
                                if is_orange {
                                    team_paints.orange.clone()
                                } else {
                                    team_paints.blue.clone()
                                }
                            },
                            _ => return,
                        };

                        player_data.loadout_paints.body = get_paint_value(&paints[0], objects);
                        player_data.loadout_paints.decal = get_paint_value(&paints[1], objects);
                        player_data.loadout_paints.wheels = get_paint_value(&paints[2], objects);
                        player_data.loadout_paints.boost = get_paint_value(&paints[3], objects);
                        player_data.loadout_paints.antenna = get_paint_value(&paints[4], objects);
                        player_data.loadout_paints.topper = get_paint_value(&paints[5], objects);
                        player_data.loadout_paints.trail = get_paint_value(&paints[14], objects);
                        player_data.loadout_paints.goal_explosion = get_paint_value(&paints[15], objects);
                        player_data.loadout_paints.banner = get_paint_value(&paints[16], objects);
                        player_data.loadout_paints.avatar_border = get_paint_value(&paints[20], objects);
                    }
                    _ => return,
                }
            }
            _ => return
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

        player_data.ping[state.frame] = None;
        player_data.ball_cam[state.frame] = None;
        match &mut player_data.time_till_power_up {
            Some(arr) => arr[state.frame] = None,
            _ => {}
        }
    }
}


fn get_paint_value(attributes: &Vec<Product>, objects: &Vec<String>) -> u32 {
    for attr in attributes {
        let attr_name = match objects.get(attr.object_ind as usize) {
            None => continue,
            Some(attr_name) => attr_name
        };

        match attr_name.as_ref() {
            "TAGame.ProductAttribute_Painted_TA" => {
                match attr.value {
                    ProductValue::OldPaint(paint) => return paint,
                    ProductValue::NewPaint(paint) => return paint,
                    _ => continue
                }
            }
            "TAGame.ProductAttribute_UserColor_TA" => {
                match attr.value {
                    ProductValue::OldColor(color) => return color,
                    ProductValue::NewColor(color) => return color as u32,
                    _ => continue
                }
            }
            _ => continue
        }
    }
    0
}
