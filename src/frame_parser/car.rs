use crate::frame_parser::ActorHandler;
use crate::frame_parser::models::{ParsedFrameData, Demolition};
use crate::network::frame_parser::FrameState;
use crate::Attribute;

pub struct CarHandler {}

impl ActorHandler for CarHandler {
    fn create(&self, _data: &mut ParsedFrameData, _state: &mut FrameState, _actor_id: i32) {}

    fn update(&self, data: &mut ParsedFrameData, state: &mut FrameState, actor_id: i32,
              updated_attr: &String, _objects: &Vec<String>) {
        let attributes = try_opt!(state.actors.get(&actor_id));

        if updated_attr == "TAGame.Car_TA:ReplicatedDemolish" {
            if let Some(Attribute::Demolish(demolish)) = attributes.get("TAGame.Car_TA:ReplicatedDemolish") {
                if demolish.attacker.0 == -1 || demolish.victim.0 == -1 {
                    return;
                }
                let attacker_player_id = try_opt!(state.car_player_map.get(&demolish.attacker.0));
                let victim_player_id = try_opt!(state.car_player_map.get(&demolish.victim.0));
                let valid_demo = add_demo(Demolition {
                    attacker_player_id: attacker_player_id.clone(),
                    victim_player_id: victim_player_id.clone(),
                    attack_velocity: demolish.attack_velocity.clone(),
                    victim_velocity: demolish.victim_velocity.clone(),
                    frame_number: state.frame.clone(),
                }, data);

                if !valid_demo { return; }

                if state.should_collect_stats() {

                    // check if the demoed player had an inactive rumble item
                    data.player_data.get_mut(&victim_player_id).map(|player_data| {
                        if !player_data.power_up_active[state.frame - 1].unwrap_or(true) {
                            player_data.rumble_item_events.last_mut().map(|event| {
                                if event.frame_use.is_none() {
                                    event.demoed = true;
                                }
                            });
                        }
                    });
                }
            }
            return;
        }

        let player_actor_id = match attributes.get("Engine.Pawn:PlayerReplicationInfo") {
            Some(Attribute::ActiveActor(actor)) => actor.actor.0.clone(),
            _ => return,
        };
        let player_data = try_opt!(data.player_data.get_mut(&player_actor_id));
        state.car_player_map.insert(actor_id, player_actor_id);

        match updated_attr.as_ref() {
            "TAGame.RBActor_TA:ReplicatedRBState" =>
                if let Some(Attribute::RigidBody(rigid_body)) = attributes.get("TAGame.RBActor_TA:ReplicatedRBState") {
                    player_data.rigid_body.add_rigid_body(state.frame, rigid_body, true);
                },
            "TAGame.Vehicle_TA:ReplicatedThrottle" =>
                if let Some(Attribute::Byte(byte)) = attributes.get("TAGame.Vehicle_TA:ReplicatedThrottle") {
                    player_data.throttle[state.frame] = Some(byte.clone());
                },
            "TAGame.Vehicle_TA:ReplicatedSteer" =>
                if let Some(Attribute::Byte(byte)) = attributes.get("TAGame.Vehicle_TA:ReplicatedSteer") {
                    player_data.steer[state.frame] = Some(byte.clone());
                }
            "TAGame.Vehicle_TA:bReplicatedHandbrake" =>
                if let Some(Attribute::Boolean(bool)) = attributes.get("TAGame.Vehicle_TA:bReplicatedHandbrake") {
                    player_data.handbrake[state.frame] = Some(bool.clone());
                }
            "TAGame.Car_TA:TeamPaint" =>
                if let Some(Attribute::TeamPaint(team_paint)) = attributes.get("TAGame.Car_TA:TeamPaint") {
                    player_data.primary_color = Some(team_paint.primary_color);
                    player_data.accent_color = Some(team_paint.accent_color);
                    player_data.primary_finish = team_paint.primary_finish;
                    player_data.accent_finish = team_paint.accent_finish;
                }
            _ => return,
        }
    }

    fn destroy(&self, data: &mut ParsedFrameData, state: &mut FrameState, actor_id: i32) {
        let player_actor_id = try_opt!(state.car_player_map.get(&actor_id));
        let player_data = try_opt!(data.player_data.get_mut(&player_actor_id));

        player_data.steer[state.frame] = None;
        player_data.throttle[state.frame] = None;
        player_data.handbrake[state.frame] = None;
        player_data.rigid_body.destroy_frame(state.frame);
    }
}

fn add_demo(demolish: Demolition, data: &mut ParsedFrameData) -> bool {
    // check for duplicate demos
    if data.demos.iter().any(|demo| {
        demo.victim_player_id == demolish.victim_player_id &&
            demo.attacker_player_id == demolish.attacker_player_id &&
            demo.attack_velocity == demolish.attack_velocity &&
            demo.victim_velocity == demolish.victim_velocity
    }) { return false; }

    data.demos.push(demolish);
    true
}
