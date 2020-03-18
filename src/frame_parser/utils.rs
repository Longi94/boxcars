use std::collections::HashMap;
use crate::Attribute;
use crate::attributes::RemoteId;
use crate::network::frame_parser::FrameState;

pub fn get_remote_id(actor: &HashMap<String, Attribute>, attribute_name: &str) -> Option<String> {
    let remote_id = match actor.get(attribute_name) {
        Some(Attribute::UniqueId(unique_id)) => unique_id.remote_id.clone(),
        Some(Attribute::PartyLeader(Some(leader))) => leader.remote_id.clone(),
        _ => return None,
    };

    match remote_id {
        RemoteId::PlayStation(ps_id) => Some(ps_id.online_id.to_string()),
        RemoteId::PsyNet(psy_net_id) => Some(psy_net_id.online_id.to_string()),
        RemoteId::SplitScreen(id) => Some(id.to_string()),
        RemoteId::Steam(id) => Some(id.to_string()),
        RemoteId::Switch(switch_id) => Some(switch_id.online_id.to_string()),
        RemoteId::Xbox(id) => Some(id.to_string()),
        RemoteId::QQ(id) => Some(id.to_string()),
    }
}

pub fn get_cars_player_actor_id(attributes: &HashMap<String, Attribute>, state: &FrameState) -> Option<i32> {
    let car_actor_id = match attributes.get("TAGame.CarComponent_TA:Vehicle") {
        Some(Attribute::ActiveActor(actor)) => if actor.actor.0 != -1 {
            actor.actor.0
        } else {
            return None
        },
        _ => return None,
    };

    match state.actors.get(&car_actor_id) {
        Some(attributes) => match attributes.get("Engine.Pawn:PlayerReplicationInfo") {
            Some(Attribute::ActiveActor(actor)) => if actor.actor.0 != -1 {
                Some(actor.actor.0)
            } else {
                None
            },
            _ => None,
        },
        _ => None,
    }
}
