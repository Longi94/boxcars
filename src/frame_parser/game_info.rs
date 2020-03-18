use crate::frame_parser::ActorHandler;
use crate::frame_parser::models::ParsedFrameData;
use crate::network::frame_parser::FrameState;
use crate::Attribute;

pub struct GameInfoHandler {}

impl ActorHandler for GameInfoHandler {
    fn create(&self, _data: &mut ParsedFrameData, _state: &mut FrameState, _actor_id: i32) {}

    fn update(&self, data: &mut ParsedFrameData, state: &mut FrameState, actor_id: i32,
              updated_attr: &String, _objects: &Vec<String>) {
        let attributes = match state.actors.get(&actor_id) {
            Some(attributes) => attributes,
            _ => return,
        };

        match updated_attr.as_ref() {
            "ProjectX.GRI_X:GameServerID" => {
                match attributes.get("ProjectX.GRI_X:GameServerID") {
                    Some(Attribute::QWord(id)) => data.game_info.server_id = Some(id.clone()),
                    _ => return,
                }
            }
            "Engine.GameReplicationInfo:ServerName" => {
                match attributes.get("Engine.GameReplicationInfo:ServerName") {
                    Some(Attribute::String(name)) => data.game_info.server_name = Some(name.clone()),
                    _ => return,
                }
            }
            "ProjectX.GRI_X:MatchGUID" => {
                match attributes.get("ProjectX.GRI_X:MatchGUID") {
                    Some(Attribute::String(guid)) => data.game_info.match_guid = Some(guid.clone()),
                    _ => return,
                }
            }
            "ProjectX.GRI_X:ReplicatedGamePlaylist" => {
                match attributes.get("ProjectX.GRI_X:ReplicatedGamePlaylist") {
                    Some(Attribute::Int(playlist)) => data.game_info.playlist = Some(playlist.clone()),
                    _ => return,
                }
            }
            "ProjectX.GRI_X:ReplicatedGameMutatorIndex" => {
                match attributes.get("ProjectX.GRI_X:ReplicatedGameMutatorIndex") {
                    Some(Attribute::Int(index)) => data.game_info.mutator_index = index.clone(),
                    _ => return,
                }
            }
            _ => return,
        }
    }

    fn destroy(&self, _data: &mut ParsedFrameData, _state: &mut FrameState, _actor_id: i32) {
    }
}
