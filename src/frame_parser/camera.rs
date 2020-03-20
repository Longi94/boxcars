use crate::frame_parser::ActorHandler;
use crate::frame_parser::models::ParsedFrameData;
use crate::network::frame_parser::FrameState;
use crate::Attribute;
use std::ops::Deref;

pub struct CameraSettingsHandler {}

impl ActorHandler for CameraSettingsHandler {
    fn create(&self, _data: &mut ParsedFrameData, _state: &mut FrameState, _actor_id: i32) {}

    fn update(&self, data: &mut ParsedFrameData, state: &mut FrameState, actor_id: i32,
              updated_attr: &String, _objects: &Vec<String>)  {
        let attributes = try_opt!(state.actors.get(&actor_id));

        let player_actor_id = match attributes.get("TAGame.CameraSettingsActor_TA:PRI") {
            Some(Attribute::ActiveActor(actor)) => actor.actor.0.clone(),
            _ => return,
        };

        let player_data = try_opt!(data.player_data.get_mut(&player_actor_id));

        match updated_attr.as_ref() {
            "TAGame.CameraSettingsActor_TA:bUsingSecondaryCamera" => {
                if let Some(Attribute::Boolean(bool)) = attributes.get("TAGame.CameraSettingsActor_TA:bUsingSecondaryCamera") {
                    player_data.ball_cam[state.frame] = Some(bool.clone());
                }
            }
            "TAGame.CameraSettingsActor_TA:ProfileSettings" => {
                if let Some(Attribute::CamSettings(settings)) = attributes.get("TAGame.CameraSettingsActor_TA:ProfileSettings") {
                    player_data.camera_settings = Some(settings.deref().clone());
                }
            }
            _ => return,
        }
    }

    fn destroy(&self, _data: &mut ParsedFrameData, _state: &mut FrameState, _actor_id: i32) {
    }
}
