use crate::frame_parser::ActorHandler;
use crate::frame_parser::models::ParsedFrameData;
use crate::network::frame_parser::FrameState;
use crate::Attribute;

pub struct GameInfoHandler {}

impl ActorHandler for GameInfoHandler {
    fn create(&self, _data: &mut ParsedFrameData, _state: &mut FrameState, _actor_id: i32) {}

    fn update(&self, data: &mut ParsedFrameData, state: &mut FrameState, actor_id: i32,
              updated_attr: &String, _objects: &Vec<String>) {
        let attributes = try_opt!(state.actors.get(&actor_id));
        match updated_attr.as_ref() {
            "ProjectX.GRI_X:GameServerID" => {
                if let Some(Attribute::QWord(id)) = attributes.get("ProjectX.GRI_X:GameServerID") {
                    data.game_info.server_id = Some(id.clone());
                }
            }
            "Engine.GameReplicationInfo:ServerName" => {
                if let Some(Attribute::String(name)) = attributes.get("Engine.GameReplicationInfo:ServerName") {
                    data.game_info.server_name = Some(name.clone());
                }
            }
            "ProjectX.GRI_X:MatchGUID" => {
                if let Some(Attribute::String(guid)) = attributes.get("ProjectX.GRI_X:MatchGUID") {
                    data.game_info.match_guid = Some(guid.clone());
                }
            }
            "ProjectX.GRI_X:ReplicatedGamePlaylist" => {
                if let Some(Attribute::Int(playlist)) = attributes.get("ProjectX.GRI_X:ReplicatedGamePlaylist") {
                    data.game_info.playlist = Some(playlist.clone());
                }
            }
            "ProjectX.GRI_X:ReplicatedGameMutatorIndex" => {
                if let Some(Attribute::Int(index)) = attributes.get("ProjectX.GRI_X:ReplicatedGameMutatorIndex") {
                    data.game_info.mutator_index = index.clone();
                }
            }
            _ => return,
        }
    }

    fn destroy(&self, _data: &mut ParsedFrameData, _state: &mut FrameState, _actor_id: i32) {}
}
