use crate::frame_parser::ActorHandler;
use crate::frame_parser::models::{ParsedFrameData, TeamData};
use crate::network::frame_parser::FrameState;
use crate::Attribute;

pub struct TeamHandler {
    pub team: u8,
}

impl ActorHandler for TeamHandler {
    fn create(&self, data: &mut ParsedFrameData, _state: &mut FrameState, actor_id: i32) {
        if !data.team_data.contains_key(&actor_id) {
            let mut team_data = TeamData::new();
            team_data.is_orange = self.team == 1;
            data.team_data.insert(actor_id, team_data);
        }
    }

    fn update(&self, data: &mut ParsedFrameData, state: &mut FrameState, actor_id: i32,
              updated_attr: &String, _objects: &Vec<String>) {
        let attributes = try_opt!(state.actors.get(&actor_id));
        let team_data = try_opt!(data.team_data.get_mut(&actor_id));
        match updated_attr.as_ref() {
            "TAGame.Team_TA:CustomTeamName" => {
                if let Some(Attribute::String(name)) = attributes.get("TAGame.Team_TA:CustomTeamName") {
                    team_data.name = Some(name.clone());
                };
            }
            "Engine.TeamInfo:Score" => {
                if let Some(Attribute::Int(score)) = attributes.get("Engine.TeamInfo:Score") {
                    team_data.score = score.clone();
                }
            }
            _ => return
        }
    }

    fn destroy(&self, _data: &mut ParsedFrameData, _state: &mut FrameState, _actor_id: i32) {}
}
