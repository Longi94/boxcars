use bitter::BitGet;
use fnv::FnvHashMap;

use crate::errors::{FrameContext, NetworkError};
use crate::network::attributes::{AttributeDecoder};
use crate::network::{VersionTriplet};
use std::collections::HashMap;
use crate::Attribute;
use crate::frame_parser::models::ParsedFrameData;
use crate::frame_parser::{ActorHandler, get_handler};
use crate::network::frame_decoder::{FrameDecoder, DecodedFrame};
use crate::header::Header;

pub(crate) struct FrameParser<'a, 'b: 'a> {
    pub frame_decoder: &'a FrameDecoder<'a, 'b>,
    pub objects: &'a Vec<String>,
    pub header: &'a Header,
}

impl<'a, 'b> FrameParser<'a, 'b> {
    pub fn decode_frames(&self) -> Result<ParsedFrameData, NetworkError> {
        let attr_decoder = AttributeDecoder::new(self.frame_decoder.version, self.frame_decoder.product_decoder);
        let mut actors = FnvHashMap::default();
        let mut bits = BitGet::new(self.frame_decoder.body.network_data);
        let mut new_actors = Vec::new();
        let mut updated_actors = Vec::new();
        let mut deleted_actors = Vec::new();

        let mut frames_data: ParsedFrameData = ParsedFrameData::with_capacity(self.frame_decoder.frames_len);
        let mut state = FrameState::new();
        let mut actors_handlers: HashMap<i32, Box<dyn ActorHandler>> = HashMap::new();
        state.total_frames = self.frame_decoder.frames_len;

        let goal_props = self.header.properties.iter()
            .find(|&(key,_)| key == "Goals")
            .and_then(|&(_, ref prop)| prop.as_array())
            .unwrap();

        let goal_frames: Vec<usize> = goal_props.iter()
            .map(|x| x.iter()
                .find(|&(key,_)| key == "frame")
                .and_then(|&(_, ref prop)| prop.as_i32())
                .unwrap() as usize
            )
            .collect();
        let mut current_goal: usize = 0;

        while !bits.is_empty() && state.frame < self.frame_decoder.frames_len {
            let frame = self.frame_decoder
                .decode_frame(
                    &attr_decoder,
                    &mut bits,
                    &mut actors,
                    &mut new_actors,
                    &mut deleted_actors,
                    &mut updated_actors,
                )
                .map_err(|e| {
                    NetworkError::FrameError(
                        e,
                        Box::new(FrameContext {
                            objects: self.frame_decoder.body.objects.clone(),
                            object_attributes: self.frame_decoder
                                .object_ind_attributes
                                .iter()
                                .map(|(key, value)| {
                                    (
                                        *key,
                                        value
                                            .attributes
                                            .iter()
                                            .map(|(key2, value)| (*key2, value.object_id))
                                            .collect(),
                                    )
                                })
                                .collect(),
                            frames: Vec::new(),
                            actors: actors.clone(),
                            new_actors: new_actors.clone(),
                            updated_actors: updated_actors.clone(),
                        }),
                    )
                })?;

            match frame {
                DecodedFrame::EndFrame => break,
                DecodedFrame::Frame(frame) => {
                    state.delta = frame.delta;
                    state.time = frame.time;
                    frames_data.new_frame(state.frame, frame.time, frame.delta);

                    // Remove deleted actors
                    for deleted in &frame.deleted_actors {
                        match actors_handlers.remove(&deleted.0) {
                            Some(handler) => handler.destroy(&mut frames_data, &mut state, deleted.0),
                            _ => {}
                        };
                        state.actors.remove(&deleted.0);
                        state.actor_objects.remove(&deleted.0);
                    }

                    // Create new actors, get handlers if any
                    for new_actor in &frame.new_actors {
                        if actors_handlers.contains_key(&new_actor.actor_id.0) {
                            match actors_handlers.remove(&new_actor.actor_id.0) {
                                Some(handler) => handler.destroy(&mut frames_data, &mut state, new_actor.actor_id.0),
                                _ => {}
                            }
                        }

                        state.actors.insert(new_actor.actor_id.0, HashMap::new());
                        let object_name = match self.objects.get(new_actor.object_id.0 as usize) {
                            None => continue,
                            Some(object_name) => object_name
                        };
                        state.actor_objects.insert(new_actor.actor_id.0, object_name.clone());

                        let handler = match get_handler(object_name) {
                            None => continue,
                            Some(handler) => handler
                        };

                        handler.create(&mut frames_data, &mut state, new_actor.actor_id.0);
                        actors_handlers.insert(new_actor.actor_id.0, handler);
                    }

                    // Update the properties of the actors
                    for updated_actor in &frame.updated_actors {
                        match state.actors.get_mut(&updated_actor.actor_id.0) {
                            None => continue,
                            Some(attributes) => {
                                match self.objects.get(updated_actor.object_id.0 as usize) {
                                    None => continue,
                                    Some(object_name) => {
                                        attributes.insert(object_name.clone(), updated_actor.attribute.clone());
                                    }
                                };
                            }
                        }
                    }

                    // Apply the update handler to each updated property
                    for updated_actor in &frame.updated_actors {
                        let handler = match actors_handlers.get(&updated_actor.actor_id.0) {
                            None => continue,
                            Some(handler) => handler
                        };

                        let object_name = match self.objects.get(updated_actor.object_id.0 as usize) {
                            None => continue,
                            Some(object_name) => object_name
                        };

                        handler.update(&mut frames_data, &mut state, updated_actor.actor_id.0, &object_name,
                                       &self.objects);
                    }

                    if current_goal < goal_frames.len() && state.frame == goal_frames[current_goal] {
                        state.is_after_goal = true;
                        state.is_kickoff = false;
                        current_goal += 1;
                    }
                    state.frame += 1;
                }
            }
        }

        if self.frame_decoder.version >= VersionTriplet(868, 24, 10) {
            bits.read_u32()
                .ok_or_else(|| NetworkError::NotEnoughDataFor("Trailer"))?;
        }

        Ok(frames_data)
    }
}

pub struct FrameState {
    pub total_frames: usize,
    pub time: f32,
    pub delta: f32,
    pub frame: usize,
    pub actors: HashMap<i32, HashMap<String, Attribute>>,
    pub actor_objects: HashMap<i32, String>,
    pub car_player_map: HashMap<i32, i32>,

    pub is_kickoff: bool,
    pub is_after_goal: bool,
}

impl FrameState {
    pub fn new() -> Self {
        FrameState {
            total_frames: 0,
            time: 0.0,
            delta: 0.0,
            frame: 0,
            actors: HashMap::new(),
            actor_objects: HashMap::new(),
            car_player_map: HashMap::new(),
            is_kickoff: false,
            is_after_goal: true,
        }
    }

    pub fn should_collect_stats(&self) -> bool {
        !self.is_after_goal
    }
}
