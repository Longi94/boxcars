use crate::frame_parser::ActorHandler;
use crate::frame_parser::models::{ParsedFrameData, Paints, UserColors};
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
                    Some(Attribute::Int(score)) => player_data.match_score = score.clone(),
                    _ => return,
                }
            }
            "TAGame.PRI_TA:MatchGoals" => {
                match attributes.get("TAGame.PRI_TA:MatchGoals") {
                    Some(Attribute::Int(goals)) => player_data.goals = goals.clone(),
                    _ => return,
                }
            }
            "TAGame.PRI_TA:MatchAssists" => {
                match attributes.get("TAGame.PRI_TA:MatchAssists") {
                    Some(Attribute::Int(assists)) => player_data.assists = assists.clone(),
                    _ => return,
                }
            }
            "TAGame.PRI_TA:MatchSaves" => {
                match attributes.get("TAGame.PRI_TA:MatchSaves") {
                    Some(Attribute::Int(saves)) => player_data.saves = saves.clone(),
                    _ => return,
                }
            }
            "TAGame.PRI_TA:MatchShots" => {
                match attributes.get("TAGame.PRI_TA:MatchShots") {
                    Some(Attribute::Int(shots)) => player_data.shots = shots.clone(),
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
            "Engine.PlayerReplicationInfo:bBot" => {
                match attributes.get("Engine.PlayerReplicationInfo:bBot") {
                    Some(Attribute::Boolean(bot)) => player_data.is_bot = bot.clone(),
                    _ => return,
                }
            }
            "TAGame.PRI_TA:ClientLoadout" => {
                match attributes.get("TAGame.PRI_TA:ClientLoadout") {
                    Some(Attribute::Loadout(loadout)) => player_data.loadout.blue = loadout.deref().clone(),
                    _ => return,
                }
            }
            "TAGame.PRI_TA:ClientLoadouts" => {
                match attributes.get("TAGame.PRI_TA:ClientLoadouts") {
                    Some(Attribute::TeamLoadout(loadouts)) => {
                        player_data.loadout.orange = Some(loadouts.orange.clone());
                        player_data.loadout.blue = loadouts.blue.clone();
                    }
                    _ => return,
                }
            }
            "TAGame.PRI_TA:ClientLoadoutOnline" => {
                match attributes.get("TAGame.PRI_TA:ClientLoadoutOnline") {
                    Some(Attribute::LoadoutOnline(paints)) => {
                        set_paint_values(&mut player_data.loadout_paints.blue,
                                         &mut player_data.loadout_user_colors.blue, &paints, objects);
                    }
                    _ => return,
                }
            }
            "TAGame.PRI_TA:ClientLoadoutsOnline" => {
                match attributes.get("TAGame.PRI_TA:ClientLoadoutsOnline") {
                    Some(Attribute::LoadoutsOnline(team_paints)) => {
                        set_paint_values(&mut player_data.loadout_paints.blue,
                                         &mut player_data.loadout_user_colors.blue, &team_paints.blue, objects);
                        let mut orange_paints = Paints::new();
                        let mut orange_user_colors = UserColors::new();
                        set_paint_values(&mut orange_paints, &mut orange_user_colors, &team_paints.blue, objects);
                        player_data.loadout_paints.orange = Some(orange_paints);
                        player_data.loadout_user_colors.orange = Some(orange_user_colors);
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

fn set_paint_values(loadout_paints: &mut Paints, user_colors: &mut UserColors, paints: &Vec<Vec<Product>>, objects: &Vec<String>) {
    loadout_paints.body = get_paint_value(&paints[0], objects);
    loadout_paints.decal = get_paint_value(&paints[1], objects);
    loadout_paints.wheels = get_paint_value(&paints[2], objects);
    loadout_paints.rocket_trail = get_paint_value(&paints[3], objects);
    loadout_paints.antenna = get_paint_value(&paints[4], objects);
    loadout_paints.topper = get_paint_value(&paints[5], objects);
    loadout_paints.trail = get_paint_value(&paints[14], objects);
    loadout_paints.goal_explosion = get_paint_value(&paints[15], objects);
    loadout_paints.banner = get_paint_value(&paints[16], objects);
    loadout_paints.avatar_border = get_paint_value(&paints[20], objects);
    user_colors.banner = get_user_color_value(&paints[16], objects);
    user_colors.avatar_border = get_user_color_value(&paints[20], objects);
}

fn get_paint_value(attributes: &Vec<Product>, objects: &Vec<String>) -> Option<u32> {
    for attr in attributes {
        let attr_name = match objects.get(attr.object_ind as usize) {
            None => continue,
            Some(attr_name) => attr_name
        };

        match attr_name.as_ref() {
            "TAGame.ProductAttribute_Painted_TA" => {
                match attr.value {
                    ProductValue::OldPaint(paint) => return Some(paint),
                    ProductValue::NewPaint(paint) => return Some(paint),
                    _ => continue
                }
            }
            _ => continue
        }
    }
    None
}

fn get_user_color_value(attributes: &Vec<Product>, objects: &Vec<String>) -> Option<u32> {
    for attr in attributes {
        let attr_name = match objects.get(attr.object_ind as usize) {
            None => continue,
            Some(attr_name) => attr_name
        };

        match attr_name.as_ref() {
            "TAGame.ProductAttribute_UserColor_TA" => {
                match attr.value {
                    ProductValue::OldColor(color) => return Some(color),
                    ProductValue::NewColor(color) => return Some(color as u32),
                    _ => continue
                }
            }
            _ => continue
        }
    }
    None
}
