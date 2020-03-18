use crate::frame_parser::ActorHandler;
use crate::frame_parser::models::{ParsedFrameData, DropshotData, DropshotDamageEvent, DropshotTile};
use crate::network::frame_parser::FrameState;
use crate::Attribute;

pub struct PlatformHandler {
    pub tile_id: u32,
}

impl ActorHandler for PlatformHandler {
    fn create(&self, data: &mut ParsedFrameData, state: &mut FrameState, _actor_id: i32) {
        if data.dropshot_data.is_none() {
            data.dropshot_data = Some(DropshotData::new())
        }

        let dropshot_data = data.dropshot_data.as_mut().unwrap();

        if !dropshot_data.tile_frames.contains_key(&self.tile_id) {
            let mut vec: Vec<u8> = Vec::with_capacity(state.total_frames);
            vec.resize(state.total_frames, 0);
            dropshot_data.tile_frames.insert(self.tile_id.clone(), vec);
        }
    }

    fn update(&self, data: &mut ParsedFrameData, state: &mut FrameState, actor_id: i32,
              updated_attr: &String, _objects: &Vec<String>) {
        let attributes = match state.actors.get(&actor_id) {
            Some(attributes) => attributes,
            _ => return,
        };

        let dropshot_data = data.dropshot_data.as_mut().unwrap();
        let tile_frames = dropshot_data.tile_frames.get_mut(&self.tile_id).unwrap();

        match updated_attr.as_ref() {
            "TAGame.BreakOutActor_Platform_TA:DamageState" => {
                match attributes.get("TAGame.BreakOutActor_Platform_TA:DamageState") {
                    Some(Attribute::DamageState(ds)) => {
                        tile_frames[state.frame] = ds.tile_state;

                        if state.frame > 0 && tile_frames[state.frame] > tile_frames[state.frame - 1] {
                            if !dropshot_data.damage_events.contains_key(&state.frame) {
                                dropshot_data.damage_events.insert(state.frame, DropshotDamageEvent {
                                    offender: ds.offender.0,
                                    tiles: Vec::new(),
                                });
                            }

                            dropshot_data.damage_events.get_mut(&state.frame).unwrap().tiles.push(DropshotTile {
                                tile_id: self.tile_id,
                                state: ds.tile_state,
                                direct_hit: ds.direct_hit,
                            })
                        }
                    }
                    _ => return,
                }
            }
            _ => return,
        }
    }

    fn destroy(&self, data: &mut ParsedFrameData, state: &mut FrameState, _actor_id: i32) {
        data.dropshot_data.as_mut().unwrap().tile_frames.get_mut(&self.tile_id).unwrap()[state.frame] = 0;
    }
}

pub fn map_tiles(id: u32) -> u32 {
    match id {
        // BLUE
        178 => 0,
        222 => 1,
        221 => 2,
        218 => 3,
        219 => 4,
        220 => 5,
        173 => 6,
        214 => 7,
        215 => 8,
        216 => 9,
        217 => 10,
        206 => 11,
        207 => 12,
        208 => 13,
        209 => 14,
        155 => 15,
        154 => 16,
        153 => 17,
        152 => 18,
        147 => 19,
        148 => 20,
        149 => 21,
        150 => 22,
        151 => 23,
        213 => 24,
        143 => 25,
        144 => 26,
        145 => 27,
        146 => 28,
        138 => 29,
        139 => 30,
        140 => 31,
        141 => 32,
        142 => 33,
        99 => 34,
        204 => 35,
        203 => 36,
        202 => 37,
        199 => 38,
        192 => 39,
        193 => 40,
        195 => 41,
        196 => 42,
        197 => 43,
        29 => 44,
        101 => 45,
        98 => 46,
        188 => 47,
        189 => 48,
        190 => 49,
        191 => 50,
        180 => 51,
        181 => 52,
        182 => 53,
        183 => 54,
        25 => 55,
        93 => 56,
        42 => 57,
        97 => 58,
        17 => 59,
        16 => 60,
        14 => 61,
        1 => 62,
        169 => 63,
        20 => 64,
        21 => 65,
        22 => 66,
        23 => 67,
        9 => 68,
        92 => 69,
        // ORANGE
        32 => 70,
        94 => 71,
        31 => 72,
        28 => 73,
        27 => 74,
        18 => 75,
        227 => 76,
        10 => 77,
        11 => 78,
        12 => 79,
        13 => 80,
        4 => 81,
        90 => 82,
        100 => 83,
        95 => 84,
        36 => 85,
        35 => 86,
        34 => 87,
        33 => 88,
        45 => 89,
        44 => 90,
        43 => 91,
        41 => 92,
        5 => 93,
        91 => 94,
        96 => 95,
        51 => 96,
        50 => 97,
        48 => 98,
        47 => 99,
        46 => 100,
        53 => 101,
        54 => 102,
        55 => 103,
        56 => 104,
        6 => 105,
        106 => 106,
        105 => 107,
        104 => 108,
        103 => 109,
        102 => 110,
        110 => 111,
        109 => 112,
        108 => 113,
        107 => 114,
        65 => 115,
        115 => 116,
        114 => 117,
        113 => 118,
        112 => 119,
        111 => 120,
        116 => 121,
        117 => 122,
        118 => 123,
        119 => 124,
        61 => 125,
        60 => 126,
        59 => 127,
        58 => 128,
        69 => 129,
        68 => 130,
        67 => 131,
        66 => 132,
        73 => 133,
        72 => 134,
        71 => 135,
        70 => 136,
        76 => 137,
        77 => 138,
        78 => 139,
        _ => 0
    }
}
