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
        let attributes = try_opt!(state.actors.get(&actor_id));
        match updated_attr.as_ref() {
            "TAGame.RBActor_TA:ReplicatedRBState" => {
                if let Some(Attribute::RigidBody(rigid_body)) = attributes.get("TAGame.RBActor_TA:ReplicatedRBState") {
                    data.ball_data.rigid_body.add_rigid_body(state.frame, rigid_body, false);
                }
            },
            "TAGame.Ball_TA:HitTeamNum" =>  {
                if let Some(Attribute::Byte(b)) = attributes.get("TAGame.Ball_TA:HitTeamNum"){
                    data.ball_data.hit_team_no[state.frame] = Some(b.clone());
                }
            },
            "TAGame.Ball_Breakout_TA:DamageIndex" => {
                if let Some(Attribute::Int(damage)) = attributes.get("TAGame.Ball_Breakout_TA:DamageIndex") {
                    let phase = data.ball_data.dropshot_phase.as_mut().unwrap();
                    phase[state.frame] = damage.clone() as u8;
                    if state.frame > 0 && phase[state.frame] > phase[state.frame - 1] {
                        if let Some(Attribute::Byte(team)) = attributes.get("TAGame.Ball_Breakout_TA:LastTeamTouch") {
                            data.dropshot_data.as_mut().unwrap().ball_events.push(DropshotBallEvent {
                                state: damage.clone() as u8,
                                frame_number: state.frame,
                                team: team.clone(),
                            })
                        }
                    }
                }
            }
            _ => return
        }
    }

    fn destroy(&self, _data: &mut ParsedFrameData, _state: &mut FrameState, _actor_id: i32) {}
}
