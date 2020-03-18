use crate::frame_parser::ActorHandler;
use crate::frame_parser::models::ParsedFrameData;
use crate::network::frame_parser::FrameState;
use crate::Attribute;
use std::ops::Deref;

pub struct CameraSettingsHandler {}

impl ActorHandler for CameraSettingsHandler {
    fn create(&self, _data: &mut ParsedFrameData, _state: &mut FrameState, _actor_id: i32) {}

    fn update(&self, data: &mut ParsedFrameData, state: &mut FrameState, actor_id: i32,
              updated_attr: &String, _objects: &Vec<String>) {
        let attributes = match state.actors.get(&actor_id) {
            None => return,
            Some(attributes) => attributes
        };

        let player_actor_id = match attributes.get("TAGame.CameraSettingsActor_TA:PRI") {
            Some(Attribute::ActiveActor(actor)) => actor.actor.0.clone(),
            _ => return,
        };

        let mut player_data = match data.player_data.get_mut(&player_actor_id) {
            Some(player_data) => player_data,
            _ => return,
        };

        match updated_attr.as_ref() {
            "TAGame.CameraSettingsActor_TA:bUsingSecondaryCamera" => {
                match attributes.get("TAGame.CameraSettingsActor_TA:bUsingSecondaryCamera") {
                    Some(Attribute::Boolean(bool)) => player_data.ball_cam.push(Some(bool.clone())),
                    _ => return,
                }
            }
            "TAGame.CameraSettingsActor_TA:ProfileSettings" => {
                match attributes.get("TAGame.CameraSettingsActor_TA:ProfileSettings") {
                    Some(Attribute::CamSettings(settings)) =>
                        player_data.camera_settings = Some(settings.deref().clone()),
                    _ => return
                }
            }
            _ => return,
        }
    }
}
