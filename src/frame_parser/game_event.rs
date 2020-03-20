use crate::frame_parser::ActorHandler;
use crate::frame_parser::models::ParsedFrameData;
use crate::Attribute;
use crate::network::frame_parser::FrameState;

pub struct GameEventHandler {}

impl ActorHandler for GameEventHandler {
    fn create(&self, _: &mut ParsedFrameData, _: &mut FrameState, _: i32) {}

    fn update(&self, data: &mut ParsedFrameData, state: &mut FrameState,
              actor_id: i32, updated_attr: &String, objects: &Vec<String>) {
        let attributes = match state.actors.get(&actor_id) {
            Some(attributes) => attributes,
            _ => return,
        };

        match updated_attr.as_ref() {
            "TAGame.GameEvent_Soccar_TA:bOverTime" => {
                match attributes.get("TAGame.GameEvent_Soccar_TA:bOverTime") {
                    Some(Attribute::Boolean(b)) => data.frames_data.is_overtime[state.frame] = Some(b.clone()),
                    _ => return
                }
            }
            "TAGame.GameEvent_Soccar_TA:SecondsRemaining" => {
                match attributes.get("TAGame.GameEvent_Soccar_TA:SecondsRemaining") {
                    Some(Attribute::Int(time)) => data.frames_data.seconds_remaining[state.frame] = Some(time.clone()),
                    _ => return
                }
            }
            "TAGame.GameEvent_TA:ReplicatedGameStateTimeRemaining" => {
                match attributes.get("TAGame.GameEvent_TA:ReplicatedGameStateTimeRemaining") {
                    Some(Attribute::Int(time)) => {
                        data.frames_data.replicated_seconds_remaining[state.frame] = Some(time.clone());
                        if state.frame > 0 && time.clone() == 0 &&
                            data.frames_data.replicated_seconds_remaining[state.frame - 1].unwrap_or(0) == 3 {
                            data.frames_data.kickoff_frames.push(state.frame);
                            state.is_kickoff = true;
                            state.is_after_goal = false;
                        }
                    }
                    _ => return
                }
            }
            "TAGame.GameEvent_Soccar_TA:bBallHasBeenHit" => {
                match attributes.get("TAGame.GameEvent_Soccar_TA:bBallHasBeenHit") {
                    Some(Attribute::Boolean(b)) => {
                        data.frames_data.ball_has_been_hit[state.frame] = Some(b.clone());
                        if state.frame > 0 && b.clone() &&
                            !data.frames_data.ball_has_been_hit[state.frame - 1].unwrap_or(false) {
                            data.frames_data.first_touch_frames.push(state.frame);
                            state.is_kickoff = false;
                        }
                    }
                    _ => return
                }
            }
            "TAGame.GameEvent_Soccar_TA:SubRulesArchetype" => {
                match attributes.get("TAGame.GameEvent_Soccar_TA:SubRulesArchetype") {
                    Some(Attribute::ActiveActor(actor)) => {
                        data.game_info.rumble_mutator = Some(objects[actor.actor.0 as usize].clone())
                    }
                    _ => return
                }
            }
            _ => return
        }
    }

    fn destroy(&self, data: &mut ParsedFrameData, state: &mut FrameState, _actor_id: i32) {
        data.frames_data.is_overtime[state.frame] = None;
        data.frames_data.seconds_remaining[state.frame] = None;
        data.frames_data.replicated_seconds_remaining[state.frame] = None;
        data.frames_data.ball_has_been_hit[state.frame] = None;
    }
}
