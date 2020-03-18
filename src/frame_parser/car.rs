use crate::frame_parser::ActorHandler;
use crate::frame_parser::models::{ParsedFrameData, Demolition};
use crate::network::frame_parser::FrameState;
use crate::Attribute;

pub struct CarHandler {}

impl ActorHandler for CarHandler {
    fn create(&self, _data: &mut ParsedFrameData, _state: &mut FrameState, _actor_id: i32) {}

    fn update(&self, data: &mut ParsedFrameData, state: &mut FrameState, actor_id: i32,
              updated_attr: &String, _objects: &Vec<String>) {
        let attributes = match state.actors.get(&actor_id) {
            None => return,
            Some(attributes) => attributes
        };

        let player_actor_id = match attributes.get("Engine.Pawn:PlayerReplicationInfo") {
            Some(Attribute::ActiveActor(actor)) => actor.actor.0.clone(),
            _ => return,
        };

        if !state.car_player_map.contains_key(&actor_id) {
            state.car_player_map.insert(actor_id, player_actor_id);
        }

        match updated_attr.as_ref() {
            "TAGame.RBActor_TA:ReplicatedRBState" => match attributes.get("TAGame.RBActor_TA:ReplicatedRBState") {
                Some(Attribute::RigidBody(rigid_body)) => data.player_data.get_mut(&player_actor_id)
                    .unwrap().rigid_body.add_rigid_body(state.frame, rigid_body, true),
                _ => return
            },
            "TAGame.Vehicle_TA:ReplicatedThrottle" => match attributes.get("TAGame.Vehicle_TA:ReplicatedThrottle") {
                Some(Attribute::Byte(byte)) => data.player_data.get_mut(&player_actor_id).unwrap().
                    throttle[state.frame] = Some(byte.clone()),
                _ => return
            }
            "TAGame.Vehicle_TA:ReplicatedSteer" => match attributes.get("TAGame.Vehicle_TA:ReplicatedSteer") {
                Some(Attribute::Byte(byte)) => data.player_data.get_mut(&player_actor_id).unwrap().
                    steer[state.frame] = Some(byte.clone()),
                _ => return
            }
            "TAGame.Vehicle_TA:bReplicatedHandbrake" => match attributes.get("TAGame.Vehicle_TA:bReplicatedHandbrake") {
                Some(Attribute::Boolean(bool)) => data.player_data.get_mut(&player_actor_id).unwrap().
                    handbrake[state.frame] = Some(bool.clone()),
                _ => return
            }
            "TAGame.Car_TA:ReplicatedDemolish" => match attributes.get("TAGame.Car_TA:ReplicatedDemolish") {
                Some(Attribute::Demolish(demolish)) => {
                    if demolish.attacker.0 == -1 || demolish.victim.0 == -1 {
                        return;
                    }
                    data.demos.push(Demolition {
                        attacker_player_id: demolish.attacker.0,
                        victim_player_id: demolish.victim.0,
                        frame_number: state.frame.clone(),
                    })
                }
                _ => return,
            }
            "TAGame.Car_TA:TeamPaint" => match attributes.get("TAGame.Car_TA:TeamPaint") {
                Some(Attribute::TeamPaint(team_paint)) => {
                    let player_data = data.player_data.get_mut(&player_actor_id).unwrap();
                    player_data.primary_color = Some(team_paint.primary_color);
                    player_data.accent_color = Some(team_paint.accent_color);
                    player_data.primary_finish = Some(team_paint.primary_finish);
                    player_data.accent_finish = Some(team_paint.accent_finish);
                }
                _ => return,
            }
            _ => return,
        }
    }
}
