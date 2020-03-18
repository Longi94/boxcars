use crate::frame_parser::ActorHandler;
use crate::frame_parser::models::{ParsedFrameData, TeamData};
use crate::network::frame_parser::FrameState;
use crate::Attribute;

pub struct TeamHandler {
    pub team: u8,
}

impl ActorHandler for TeamHandler {
    fn create(&self, data: &mut ParsedFrameData, _state: &mut FrameState, actor_id: i32) {
        if data.team_data.contains_key(&actor_id) {
            return;
        }
        let mut team_data = TeamData::new();
        team_data.is_orange = self.team == 1;
        data.team_data.insert(actor_id, team_data);
    }

    fn update(&self, data: &mut ParsedFrameData, state: &mut FrameState, actor_id: i32,
              updated_attr: &String, _objects: &Vec<String>) {
        let attributes = match state.actors.get(&actor_id) {
            Some(attributes) => attributes,
            _ => return,
        };

        let mut team_data = match data.team_data.get_mut(&actor_id) {
            Some(team_data) => team_data,
            _ => return,
        };

        match updated_attr.as_ref() {
            "TAGame.Team_TA:CustomTeamName" => {
                match attributes.get("TAGame.Team_TA:CustomTeamName") {
                    Some(Attribute::String(name)) => team_data.name = Some(name.clone()),
                    _ => return,
                };
            }
            "Engine.TeamInfo:Score" => {
                match attributes.get("Engine.TeamInfo:Score") {
                    Some(Attribute::Int(score)) => team_data.score = score.clone(),
                    _ => return,
                }
            }
            _ => return
        }
    }
}
