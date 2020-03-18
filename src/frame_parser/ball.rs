use crate::frame_parser::ActorHandler;
use crate::frame_parser::models::{BallType, ParsedFrameData, DropshotData, DropshotBallEvent};
use crate::network::frame_parser::FrameState;
use crate::Attribute;

pub struct BallHandler {
    pub ball_type: BallType
}

impl ActorHandler for BallHandler {
    fn create(&self, data: &mut ParsedFrameData, state: &mut FrameState, _: i32) {
        data.ball_data.ball_type = self.ball_type as i32;
        if self.ball_type == BallType::Breakout {
            if data.ball_data.dropshot_phase.is_none() {
                let mut vec: Vec<u8> = Vec::with_capacity(state.total_frames);
                vec.resize(state.total_frames, 0);
                data.ball_data.dropshot_phase = Some(vec);
            }
            if data.dropshot_data.is_none() {
                data.dropshot_data = Some(DropshotData::new())
            }
        }
    }

    fn update(&self, data: &mut ParsedFrameData, state: &mut FrameState, actor_id: i32,
              updated_attr: &String, _: &Vec<String>) {
        let attributes = match state.actors.get(&actor_id) {
            Some(attributes) => attributes,
            _ => return,
        };

        match updated_attr.as_ref() {
            "TAGame.RBActor_TA:ReplicatedRBState" => match attributes.get("TAGame.RBActor_TA:ReplicatedRBState") {
                Some(Attribute::RigidBody(rigid_body)) =>
                    data.ball_data.rigid_body.add_rigid_body(state.frame, rigid_body, false),
                _ => return
            },
            "TAGame.Ball_TA:HitTeamNum" => match attributes.get("TAGame.Ball_TA:HitTeamNum") {
                Some(Attribute::Byte(b)) => data.ball_data.hit_team_no[state.frame] = Some(b.clone()),
                _ => return
            },
            "TAGame.Ball_Breakout_TA:DamageIndex" => match attributes.get("TAGame.Ball_Breakout_TA:DamageIndex") {
                Some(Attribute::Int(damage)) => {
                    let phase = data.ball_data.dropshot_phase.as_mut().unwrap();
                    phase[state.frame] = damage.clone() as u8;
                    if state.frame > 0 && phase[state.frame] > phase[state.frame - 1] {
                        let team = match attributes.get("TAGame.Ball_Breakout_TA:LastTeamTouch") {
                            Some(Attribute::Byte(team)) => team,
                            _ => return,
                        };

                        data.dropshot_data.as_mut().unwrap().ball_events.push(DropshotBallEvent {
                            state: damage.clone() as u8,
                            frame_number: state.frame,
                            team: team.clone(),
                        })
                    }
                }
                _ => return
            }
            _ => return
        }
    }

    fn destroy(&self, _data: &mut ParsedFrameData, _state: &mut FrameState, _actor_id: i32) {}
}
