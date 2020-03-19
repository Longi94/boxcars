use bitter::BitGet;
use fnv::FnvHashMap;

use crate::errors::{AttributeError, FrameContext, FrameError, NetworkError};
use crate::network::attributes::{AttributeDecoder, ProductValueDecoder};
use crate::network::models::{
    ActorId, Frame, NewActor, ObjectId, SpawnTrajectory, StreamId, Trajectory, UpdatedAttribute,
};
use crate::network::{CacheInfo, VersionTriplet};
use crate::parser::ReplayBody;
use std::collections::HashMap;
use crate::Attribute;
use crate::frame_parser::models::ParsedFrameData;
use crate::frame_parser::{ActorHandler, get_handler};

pub(crate) struct FrameParser<'a, 'b: 'a> {
    pub frames_len: usize,
    pub product_decoder: ProductValueDecoder,
    pub max_channels: u32,
    pub channel_bits: i32,
    pub body: &'a ReplayBody<'b>,
    pub spawns: &'a Vec<SpawnTrajectory>,
    pub object_ind_attributes: FnvHashMap<ObjectId, CacheInfo<'a>>,
    pub version: VersionTriplet,
    pub is_lan: bool,
    pub objects: &'a Vec<String>,
}

#[derive(Debug)]
enum DecodedFrame {
    EndFrame,
    Frame(Frame),
}

impl<'a, 'b> FrameParser<'a, 'b> {
    fn parse_new_actor(
        &self,
        mut bits: &mut BitGet<'_>,
        actor_id: ActorId,
    ) -> Result<NewActor, FrameError> {
        if_chain! {
            if let Some(name_id) =
                if self.version >= VersionTriplet(868, 14, 0) && !self.is_lan {
                    bits.read_i32().map(Some)
                } else {
                    Some(None)
                };

            if let Some(_unused) = bits.read_bit();
            if let Some(object_id) = bits.read_i32().map(ObjectId);
            let spawn = self.spawns.get(usize::from(object_id))
                .ok_or_else(|| FrameError::ObjectIdOutOfRange {obj: object_id})?;

            if let Some(traj) = Trajectory::from_spawn(&mut bits, *spawn, self.version.net_version());
            then {
                Ok(NewActor {
                    actor_id,
                    name_id,
                    object_id,
                    initial_trajectory: traj
                })
            } else {
                Err(FrameError::NotEnoughDataFor("New Actor"))
            }
        }
    }

    fn decode_frame(
        &self,
        attr_decoder: &AttributeDecoder,
        mut bits: &mut BitGet<'_>,
        actors: &mut FnvHashMap<ActorId, ObjectId>,
        new_actors: &mut Vec<NewActor>,
        deleted_actors: &mut Vec<ActorId>,
        updated_actors: &mut Vec<UpdatedAttribute>,
    ) -> Result<DecodedFrame, FrameError> {
        let time = bits
            .read_f32()
            .ok_or_else(|| FrameError::NotEnoughDataFor("Time"))?;

        if time < 0.0 || (time > 0.0 && time < 1e-10) {
            return Err(FrameError::TimeOutOfRange { time });
        }

        let delta = bits
            .read_f32()
            .ok_or_else(|| FrameError::NotEnoughDataFor("Delta"))?;

        if delta < 0.0 || (delta > 0.0 && delta < 1e-10) {
            return Err(FrameError::DeltaOutOfRange { delta });
        }

        if time == 0.0 && delta == 0.0 {
            return Ok(DecodedFrame::EndFrame);
        }

        while bits
            .read_bit()
            .ok_or_else(|| FrameError::NotEnoughDataFor("Actor data"))?
        {
            let actor_id = bits
                .read_bits_max_computed(self.channel_bits, self.max_channels)
                .map(|x| ActorId(x as i32))
                .ok_or_else(|| FrameError::NotEnoughDataFor("Actor Id"))?;

            // alive
            if bits
                .read_bit()
                .ok_or_else(|| FrameError::NotEnoughDataFor("Is actor alive"))?
            {
                // new
                if bits
                    .read_bit()
                    .ok_or_else(|| FrameError::NotEnoughDataFor("Is new actor"))?
                {
                    let actor = self.parse_new_actor(&mut bits, actor_id)?;

                    // Insert the new actor so we can keep track of it for attribute
                    // updates. It's common for an actor id to already exist, so we
                    // overwrite it.
                    actors.insert(actor.actor_id, actor.object_id);
                    new_actors.push(actor);
                } else {
                    // We'll be updating an existing actor with some attributes so we need
                    // to track down what the actor's type is
                    let object_id = actors
                        .get(&actor_id)
                        .ok_or_else(|| FrameError::MissingActor { actor: actor_id })?;

                    // Once we have the type we need to look up what attributes are
                    // available for said type
                    let cache_info =
                        self.object_ind_attributes.get(object_id).ok_or_else(|| {
                            FrameError::MissingCache {
                                actor: actor_id,
                                actor_object: *object_id,
                            }
                        })?;

                    // While there are more attributes to update for our actor:
                    while bits
                        .read_bit()
                        .ok_or_else(|| FrameError::NotEnoughDataFor("Is prop present"))?
                    {
                        // We've previously calculated the max the stream id can be for a
                        // given type and how many bits that it encompasses so use those
                        // values now
                        let stream_id = bits
                            .read_bits_max_computed(cache_info.prop_id_bits, cache_info.max_prop_id)
                            .map(|x| StreamId(x as i32))
                            .ok_or_else(|| FrameError::NotEnoughDataFor("Prop id"))?;

                        // Look the stream id up and find the corresponding attribute
                        // decoding function. Experience has told me replays that fail to
                        // parse, fail to do so here, so a large chunk is dedicated to
                        // generating an error message with context
                        let attr = cache_info.attributes.get(&stream_id).ok_or_else(|| {
                            FrameError::MissingAttribute {
                                actor: actor_id,
                                actor_object: *object_id,
                                attribute_stream: stream_id,
                            }
                        })?;

                        let attribute = attr_decoder.decode(attr.attribute, &mut bits).map_err(
                            |e| match e {
                                AttributeError::Unimplemented => FrameError::MissingAttribute {
                                    actor: actor_id,
                                    actor_object: *object_id,
                                    attribute_stream: stream_id,
                                },
                                e => FrameError::AttributeError {
                                    actor: actor_id,
                                    actor_object: *object_id,
                                    attribute_stream: stream_id,
                                    error: e,
                                },
                            },
                        )?;

                        updated_actors.push(UpdatedAttribute {
                            actor_id,
                            stream_id,
                            object_id: attr.object_id,
                            attribute,
                        });
                    }
                }
            } else {
                deleted_actors.push(actor_id);
                actors.remove(&actor_id);
            }
        }

        Ok(DecodedFrame::Frame(Frame {
            time,
            delta,
            new_actors: new_actors.drain(..).collect(),
            deleted_actors: deleted_actors.drain(..).collect(),
            updated_actors: updated_actors.drain(..).collect(),
        }))
    }

    pub fn decode_frames(&self) -> Result<ParsedFrameData, NetworkError> {
        let attr_decoder = AttributeDecoder::new(self.version, self.product_decoder);
        let mut actors = FnvHashMap::default();
        let mut bits = BitGet::new(self.body.network_data);
        let mut new_actors = Vec::new();
        let mut updated_actors = Vec::new();
        let mut deleted_actors = Vec::new();

        let mut frames_data: ParsedFrameData = ParsedFrameData::with_capacity(self.frames_len);
        let mut state = FrameState::new();
        let mut actors_handlers: HashMap<i32, Box<dyn ActorHandler>> = HashMap::new();
        state.total_frames = self.frames_len;

        while !bits.is_empty() && state.frame < self.frames_len {
            let frame = self
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
                            objects: self.body.objects.clone(),
                            object_attributes: self
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

                    state.frame += 1;
                }
            }
        }

        if self.version >= VersionTriplet(868, 24, 10) {
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
        }
    }
}
