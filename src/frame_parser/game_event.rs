use crate::frame_parser::ActorHandler;
use crate::frame_parser::models::ParsedFrameData;
use crate::Attribute;
use crate::network::frame_parser::FrameState;

pub struct GameEventHandler {}

impl ActorHandler for GameEventHandler {
    fn create(&self, _: &mut ParsedFrameData, _: &mut FrameState, _: i32) {}

    fn update(&self, data: &mut ParsedFrameData, state: &mut FrameState,
              actor_id: i32, updated_attr: &String, objects: &Vec<String>) {
        if let Some(attributes) = state.actors.get(&actor_id) {
            match updated_attr.as_ref() {
                "TAGame.GameEvent_Soccar_TA:bOverTime" => {
                    if let Some(Attribute::Boolean(b)) = attributes.get("TAGame.GameEvent_Soccar_TA:bOverTime") {
                        data.frames_data.is_overtime[state.frame] = Some(b.clone());
                    }
                }
                "TAGame.GameEvent_Soccar_TA:SecondsRemaining" => {
                    if let Some(Attribute::Int(time)) = attributes.get("TAGame.GameEvent_Soccar_TA:SecondsRemaining") {
                        data.frames_data.seconds_remaining[state.frame] = Some(time.clone());
                    }
                }
                "TAGame.GameEvent_TA:ReplicatedGameStateTimeRemaining" => {
                    if let Some(Attribute::Int(time)) = attributes.get("TAGame.GameEvent_TA:ReplicatedGameStateTimeRemaining") {
                        data.frames_data.replicated_seconds_remaining[state.frame] = Some(time.clone());
                        if state.frame > 0 && time.clone() == 0 &&
                            data.frames_data.replicated_seconds_remaining[state.frame - 1].unwrap_or(0) == 3 {
                            data.frames_data.kickoff_frames.push(state.frame);
                            state.is_kickoff = true;
                            state.is_after_goal = false;
                        }
                    }
                }
                "TAGame.GameEvent_Soccar_TA:bBallHasBeenHit" => {
                    if let Some(Attribute::Boolean(b)) = attributes.get("TAGame.GameEvent_Soccar_TA:bBallHasBeenHit") {
                        data.frames_data.ball_has_been_hit[state.frame] = Some(b.clone());
                        if state.frame > 0 && b.clone() &&
                            !data.frames_data.ball_has_been_hit[state.frame - 1].unwrap_or(false) {
                            data.frames_data.first_touch_frames.push(state.frame);
                            state.is_kickoff = false;
                        }
                    }
                }
                "TAGame.GameEvent_Soccar_TA:SubRulesArchetype" => {
                    if let Some(Attribute::ActiveActor(actor)) = attributes.get("TAGame.GameEvent_Soccar_TA:SubRulesArchetype") {
                        data.game_info.rumble_mutator = Some(objects[actor.actor.0 as usize].clone());
                    }
                }
                _ => return
            }
        }
    }

    fn destroy(&self, data: &mut ParsedFrameData, state: &mut FrameState, _actor_id: i32) {
        data.frames_data.is_overtime[state.frame] = None;
        data.frames_data.seconds_remaining[state.frame] = None;
        data.frames_data.replicated_seconds_remaining[state.frame] = None;
        data.frames_data.ball_has_been_hit[state.frame] = None;
    }
}
