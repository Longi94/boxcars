use crate::frame_parser::ActorHandler;
use crate::frame_parser::models::{BallType, ParsedFrameData};
use crate::network::frame_parser::FrameState;
use crate::Attribute;

pub struct BallHandler {
    pub ball_type: BallType
}

impl ActorHandler for BallHandler {
    fn create(&self, data: &mut ParsedFrameData, _: &mut FrameState, _: i32) {
        data.ball_data.ball_type = self.ball_type;
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
            _ => return
        }
    }

    fn destroy(&self, _data: &mut ParsedFrameData, _state: &mut FrameState, _actor_id: i32) {
    }
}
