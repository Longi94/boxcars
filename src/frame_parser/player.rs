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
        let attributes = try_opt!(state.actors.get(&actor_id));
        let player_data = try_opt!(data.player_data.get_mut(&actor_id));
        match updated_attr.as_ref() {
            "Engine.PlayerReplicationInfo:Team" => {
                if let Some(Attribute::ActiveActor(actor)) = attributes.get("Engine.PlayerReplicationInfo:Team") {
                    if actor.actor.0 != -1 {
                        player_data.team_actor = actor.actor.0;
                    }
                }
            }
            "Engine.PlayerReplicationInfo:PlayerName" => {
                if let Some(Attribute::String(name)) = attributes.get("Engine.PlayerReplicationInfo:PlayerName") {
                    player_data.name = Some(name.clone());
                }
            }
            "Engine.PlayerReplicationInfo:UniqueId" => {
                player_data.remote_id = get_remote_id(attributes, "Engine.PlayerReplicationInfo:UniqueId");
            }
            "Engine.PlayerReplicationInfo:Ping" => {
                if let Some(Attribute::Byte(ping)) = attributes.get("Engine.PlayerReplicationInfo:Ping") {
                    player_data.ping[state.frame] = Some(ping.clone());
                }
            }
            "TAGame.CameraSettingsActor_TA:bUsingSecondaryCamera" => {
                if let Some(Attribute::Boolean(ball_cam)) = attributes.get("TAGame.CameraSettingsActor_TA:bUsingSecondaryCamera") {
                    player_data.ball_cam[state.frame] = Some(ball_cam.clone());
                }
            }
            "TAGame.PRI_TA:TimeTillItem" => {
                if player_data.time_till_power_up.is_none() {
                    let mut vec: Vec<Option<i32>> = Vec::with_capacity(state.total_frames);
                    vec.resize(state.total_frames, None);
                    player_data.time_till_power_up = Some(vec);
                }

                if let Some(Attribute::Int(time)) = attributes.get("TAGame.PRI_TA:TimeTillItem") {
                    player_data.time_till_power_up.as_mut()
                        .unwrap()[state.frame] = Some(time.clone());
                }
            }
            "TAGame.PRI_TA:PartyLeader" => {
                let leader_id = try_opt!(get_remote_id(attributes, "TAGame.PRI_TA:PartyLeader"));

                player_data.party_leader = Some(leader_id.clone());

                if !data.parties.contains_key(&leader_id) {
                    data.parties.insert(leader_id.clone(), HashSet::new());
                }

                if let Some(id) = get_remote_id(attributes, "Engine.PlayerReplicationInfo:UniqueId") {
                    data.parties.get_mut(&leader_id).unwrap().insert(id);
                }
            }
            "TAGame.PRI_TA:MatchScore" => {
                if let Some(Attribute::Int(score)) = attributes.get("TAGame.PRI_TA:MatchScore") {
                    player_data.match_score = score.clone();
                }
            }
            "TAGame.PRI_TA:MatchGoals" => {
                if let Some(Attribute::Int(goals)) = attributes.get("TAGame.PRI_TA:MatchGoals") {
                    player_data.goals = goals.clone();
                }
            }
            "TAGame.PRI_TA:MatchAssists" => {
                if let Some(Attribute::Int(assists)) = attributes.get("TAGame.PRI_TA:MatchAssists") {
                    player_data.assists = assists.clone();
                }
            }
            "TAGame.PRI_TA:MatchSaves" => {
                if let Some(Attribute::Int(saves)) = attributes.get("TAGame.PRI_TA:MatchSaves") {
                    player_data.saves = saves.clone();
                }
            }
            "TAGame.PRI_TA:MatchShots" => {
                if let Some(Attribute::Int(shots)) = attributes.get("TAGame.PRI_TA:MatchShots") {
                    player_data.shots = shots.clone();
                }
            }
            "TAGame.PRI_TA:Title" => {
                if let Some(Attribute::Int(title)) = attributes.get("TAGame.PRI_TA:Title") {
                    player_data.title = Some(title.clone());
                }
            }
            "TAGame.PRI_TA:TotalXP" => {
                if let Some(Attribute::Int(total_xp)) = attributes.get("TAGame.PRI_TA:TotalXP") {
                    player_data.total_xp = Some(total_xp.clone());
                }
            }
            "TAGame.PRI_TA:SteeringSensitivity" => {
                if let Some(Attribute::Float(sens)) = attributes.get("TAGame.PRI_TA:SteeringSensitivity") {
                    player_data.steering_sensitivity = Some(sens.clone());
                }
            }
            "Engine.PlayerReplicationInfo:bBot" => {
                if let Some(Attribute::Boolean(bot)) = attributes.get("Engine.PlayerReplicationInfo:bBot") {
                    player_data.is_bot = bot.clone();
                }
            }
            "TAGame.PRI_TA:ClientLoadout" => {
                if let Some(Attribute::Loadout(loadout)) = attributes.get("TAGame.PRI_TA:ClientLoadout") {
                    player_data.loadout.blue = loadout.deref().clone();
                }
            }
            "TAGame.PRI_TA:ClientLoadouts" => {
                if let Some(Attribute::TeamLoadout(loadouts)) = attributes.get("TAGame.PRI_TA:ClientLoadouts") {
                    player_data.loadout.orange = Some(loadouts.orange.clone());
                    player_data.loadout.blue = loadouts.blue.clone();
                }
            }
            "TAGame.PRI_TA:ClientLoadoutOnline" => {
                if let Some(Attribute::LoadoutOnline(paints)) = attributes.get("TAGame.PRI_TA:ClientLoadoutOnline") {
                    set_paint_values(&mut player_data.loadout_paints.blue,
                                     &mut player_data.loadout_user_colors.blue, &paints, objects);
                }
            }
            "TAGame.PRI_TA:ClientLoadoutsOnline" => {
                if let Some(Attribute::LoadoutsOnline(team_paints)) = attributes.get("TAGame.PRI_TA:ClientLoadoutsOnline") {
                    set_paint_values(&mut player_data.loadout_paints.blue,
                                     &mut player_data.loadout_user_colors.blue, &team_paints.blue, objects);
                    let mut orange_paints = Paints::new();
                    let mut orange_user_colors = UserColors::new();
                    set_paint_values(&mut orange_paints, &mut orange_user_colors, &team_paints.blue, objects);
                    player_data.loadout_paints.orange = Some(orange_paints);
                    player_data.loadout_user_colors.orange = Some(orange_user_colors);
                }
            }
            _ => return
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
                player_data.ping[state.frame] = None;
                player_data.ball_cam[state.frame] = None;
                player_data.time_till_power_up.as_mut().map(|arr| arr[state.frame] = None);
            }
        }
    }
}

fn set_paint_values(loadout_paints: &mut Paints, user_colors: &mut UserColors, paints: &Vec<Vec<Product>>, objects: &Vec<String>) {
    loadout_paints.body = get_paint_value(&paints, 0, objects);
    loadout_paints.decal = get_paint_value(&paints, 1, objects);
    loadout_paints.wheels = get_paint_value(&paints, 2, objects);
    loadout_paints.rocket_trail = get_paint_value(&paints, 3, objects);
    loadout_paints.antenna = get_paint_value(&paints, 4, objects);
    loadout_paints.topper = get_paint_value(&paints, 5, objects);
    loadout_paints.trail = get_paint_value(&paints, 14, objects);
    loadout_paints.goal_explosion = get_paint_value(&paints, 15, objects);
    loadout_paints.banner = get_paint_value(&paints, 16, objects);
    loadout_paints.avatar_border = get_paint_value(&paints, 20, objects);
    user_colors.banner = get_user_color_value(&paints, 16, objects);
    user_colors.avatar_border = get_user_color_value(&paints, 20, objects);
}

fn get_paint_value(paints: &Vec<Vec<Product>>, index: usize, objects: &Vec<String>) -> Option<u32> {
    if paints.len() <= index {
        return None;
    }

    for attr in &paints[index] {
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

fn get_user_color_value(paints: &Vec<Vec<Product>>, index: usize, objects: &Vec<String>) -> Option<u32> {
    if paints.len() <= index {
        return None;
    }

    for attr in &paints[index] {
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
