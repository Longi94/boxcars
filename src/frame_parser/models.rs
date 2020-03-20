use crate::attributes::RigidBody;
use std::f32::consts::PI;
use std::collections::{HashMap, HashSet};
use crate::{CamSettings, Vector3f};
use crate::frame_parser::boost::BOOST_PER_SECOND;
use crate::attributes::Loadout;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ParsedFrameData {
    pub game_info: GameInfo,
    pub frames_data: FramesData,
    pub ball_data: BallData,
    pub team_data: HashMap<i32, TeamData>,
    pub player_data: HashMap<i32, PlayerData>,
    pub parties: HashMap<String, HashSet<String>>,
    pub demos: Vec<Demolition>,
    pub dropshot_data: Option<DropshotData>,
}

impl ParsedFrameData {
    pub fn with_capacity(c: usize) -> Self {
        ParsedFrameData {
            game_info: GameInfo::new(),
            frames_data: FramesData::with_capacity(c),
            ball_data: BallData::with_capacity(c),
            player_data: HashMap::new(),
            team_data: HashMap::new(),
            parties: HashMap::new(),
            demos: Vec::new(),
            dropshot_data: None,
        }
    }

    pub fn new_frame(&mut self, frame: usize, time: f32, delta: f32) {
        self.frames_data.new_frame(frame, time, delta);
        self.ball_data.new_frame(frame);
        for (_, data) in &mut self.player_data {
            data.new_frame(frame, delta);
        }
        self.dropshot_data.as_mut().map(|data| data.new_frame(frame));
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GameInfo {
    pub server_id: Option<u64>,
    pub server_name: Option<String>,
    pub match_guid: Option<String>,
    pub playlist: Option<i32>,
    pub mutator_index: i32,
    pub rumble_mutator: Option<String>,
}

impl GameInfo {
    pub fn new() -> Self {
        GameInfo {
            server_id: None,
            server_name: None,
            match_guid: None,
            playlist: None,
            mutator_index: 0,
            rumble_mutator: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FramesData {
    pub time: Vec<f32>,
    pub delta: Vec<f32>,
    pub seconds_remaining: Vec<Option<i32>>,
    pub replicated_seconds_remaining: Vec<Option<i32>>,
    pub is_overtime: Vec<Option<bool>>,
    pub ball_has_been_hit: Vec<Option<bool>>,
    pub kickoff_frames: Vec<usize>,
    pub first_touch_frames: Vec<usize>,
}

impl FramesData {
    pub fn with_capacity(c: usize) -> Self {
        let mut data = FramesData {
            time: Vec::with_capacity(c),
            delta: Vec::with_capacity(c),
            seconds_remaining: Vec::with_capacity(c),
            replicated_seconds_remaining: Vec::with_capacity(c),
            is_overtime: Vec::with_capacity(c),
            ball_has_been_hit: Vec::with_capacity(c),
            kickoff_frames: Vec::new(),
            first_touch_frames: Vec::new(),
        };

        data.time.resize(c, 0.0);
        data.delta.resize(c, 0.0);
        data.seconds_remaining.resize(c, None);
        data.replicated_seconds_remaining.resize(c, None);
        data.is_overtime.resize(c, None);
        data.ball_has_been_hit.resize(c, None);

        data
    }

    pub fn new_frame(&mut self, frame: usize, time: f32, delta: f32) {
        self.time[frame] = time;
        self.delta[frame] = delta;

        if frame > 0 {
            self.seconds_remaining[frame] = self.seconds_remaining[frame - 1].clone();
            self.replicated_seconds_remaining[frame] = self.replicated_seconds_remaining[frame - 1].clone();
            self.is_overtime[frame] = self.is_overtime[frame - 1].clone();
            self.ball_has_been_hit[frame] = self.ball_has_been_hit[frame - 1].clone();
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RigidBodyFrames {
    pub pos_x: Vec<Option<f32>>,
    pub pos_y: Vec<Option<f32>>,
    pub pos_z: Vec<Option<f32>>,
    pub rot_x: Vec<Option<f32>>,
    pub rot_y: Vec<Option<f32>>,
    pub rot_z: Vec<Option<f32>>,
    pub vel_x: Vec<Option<f32>>,
    pub vel_y: Vec<Option<f32>>,
    pub vel_z: Vec<Option<f32>>,
    pub ang_vel_x: Vec<Option<f32>>,
    pub ang_vel_y: Vec<Option<f32>>,
    pub ang_vel_z: Vec<Option<f32>>,
}

impl RigidBodyFrames {
    pub fn with_capacity(c: usize) -> Self {
        let mut frames = RigidBodyFrames {
            pos_x: Vec::with_capacity(c),
            pos_y: Vec::with_capacity(c),
            pos_z: Vec::with_capacity(c),
            rot_x: Vec::with_capacity(c),
            rot_y: Vec::with_capacity(c),
            rot_z: Vec::with_capacity(c),
            vel_x: Vec::with_capacity(c),
            vel_y: Vec::with_capacity(c),
            vel_z: Vec::with_capacity(c),
            ang_vel_x: Vec::with_capacity(c),
            ang_vel_y: Vec::with_capacity(c),
            ang_vel_z: Vec::with_capacity(c),
        };

        frames.pos_x.resize(c, None);
        frames.pos_y.resize(c, None);
        frames.pos_z.resize(c, None);
        frames.rot_x.resize(c, None);
        frames.rot_y.resize(c, None);
        frames.rot_z.resize(c, None);
        frames.vel_x.resize(c, None);
        frames.vel_y.resize(c, None);
        frames.vel_z.resize(c, None);
        frames.ang_vel_x.resize(c, None);
        frames.ang_vel_y.resize(c, None);
        frames.ang_vel_z.resize(c, None);

        frames
    }

    pub fn add_rigid_body(&mut self, i: usize, rb: &RigidBody, ignore_sleeping: bool) {
        if ignore_sleeping && rb.sleeping {
            return;
        }

        self.pos_x[i] = Some(rb.location.x);
        self.pos_y[i] = Some(rb.location.y);
        self.pos_z[i] = Some(rb.location.z);

        // convert quat to euler
        let sinr = 2.0 * (rb.rotation.w * rb.rotation.x + rb.rotation.y * rb.rotation.z);
        let cosr = 1.0 - 2.0 * (rb.rotation.x * rb.rotation.x + rb.rotation.y * rb.rotation.y);
        let roll = sinr.atan2(cosr);

        let sinp = 2.0 * (rb.rotation.w * rb.rotation.y - rb.rotation.z * rb.rotation.x);

        let pitch = if sinp.abs() >= 1.0 {
            (PI / 2.0).copysign(sinp)
        } else {
            sinp.asin()
        };

        let siny = 2.0 * (rb.rotation.w * rb.rotation.z + rb.rotation.x * rb.rotation.y);
        let cosy = 1.0 - 2.0 * (rb.rotation.y * rb.rotation.y + rb.rotation.z * rb.rotation.z);
        let yaw = siny.atan2(cosy);
        self.rot_x[i] = Some(pitch);
        self.rot_y[i] = Some(yaw);
        self.rot_z[i] = Some(roll);

        rb.linear_velocity.map(|v| {
            self.vel_x[i] = Some(v.x);
            self.vel_y[i] = Some(v.y);
            self.vel_z[i] = Some(v.z);
        });

        rb.angular_velocity.map(|v| {
            self.ang_vel_x[i] = Some(v.x);
            self.ang_vel_y[i] = Some(v.y);
            self.ang_vel_z[i] = Some(v.z);
        });
    }

    pub fn new_frame(&mut self, frame: usize) {
        if frame > 0 {
            self.pos_x[frame] = self.pos_x[frame - 1].clone();
            self.pos_y[frame] = self.pos_y[frame - 1].clone();
            self.pos_z[frame] = self.pos_z[frame - 1].clone();
            self.rot_x[frame] = self.rot_x[frame - 1].clone();
            self.rot_y[frame] = self.rot_y[frame - 1].clone();
            self.rot_z[frame] = self.rot_z[frame - 1].clone();
            self.vel_x[frame] = self.vel_x[frame - 1].clone();
            self.vel_y[frame] = self.vel_y[frame - 1].clone();
            self.vel_z[frame] = self.vel_z[frame - 1].clone();
            self.ang_vel_x[frame] = self.ang_vel_x[frame - 1].clone();
            self.ang_vel_y[frame] = self.ang_vel_y[frame - 1].clone();
            self.ang_vel_z[frame] = self.ang_vel_z[frame - 1].clone();
        }
    }

    pub fn destroy_frame(&mut self, frame: usize) {
        self.pos_x[frame] = None;
        self.pos_y[frame] = None;
        self.pos_z[frame] = None;
        self.rot_x[frame] = None;
        self.rot_y[frame] = None;
        self.rot_z[frame] = None;
        self.vel_x[frame] = None;
        self.vel_y[frame] = None;
        self.vel_z[frame] = None;
        self.ang_vel_x[frame] = None;
        self.ang_vel_y[frame] = None;
        self.ang_vel_z[frame] = None;
    }
}

#[derive(Serialize, Debug, PartialEq, Copy, Clone)]
pub enum BallType {
    Unknown = 0,
    Default = 1,
    Basketball = 2,
    Puck = 3,
    Cube = 4,
    Breakout = 5,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BallData {
    pub ball_type: i32,
    pub rigid_body: RigidBodyFrames,
    pub hit_team_no: Vec<Option<u8>>,
    pub dropshot_phase: Option<Vec<u8>>,
}

impl BallData {
    pub fn with_capacity(c: usize) -> Self {
        let mut data = BallData {
            ball_type: BallType::Unknown as i32,
            rigid_body: RigidBodyFrames::with_capacity(c),
            hit_team_no: Vec::with_capacity(c),
            dropshot_phase: None,
        };
        data.hit_team_no.resize(c, None);
        data
    }

    pub fn new_frame(&mut self, frame: usize) {
        self.rigid_body.new_frame(frame);
        if frame > 0 {
            self.hit_team_no[frame] = self.hit_team_no[frame - 1].clone();
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TeamData {
    pub name: Option<String>,
    pub is_orange: bool,
    pub score: i32,
}

impl TeamData {
    pub fn new() -> Self {
        TeamData {
            name: None,
            is_orange: false,
            score: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PlayerData {
    pub remote_id: Option<String>,
    pub name: Option<String>,
    pub is_bot: bool,
    pub team_actor: i32,
    pub match_score: i32,
    pub goals: i32,
    pub assists: i32,
    pub saves: i32,
    pub shots: i32,
    pub party_leader: Option<String>,
    pub title: Option<i32>,
    pub total_xp: Option<i32>,
    pub steering_sensitivity: Option<f32>,
    pub ping: Vec<Option<u8>>,
    pub ball_cam: Vec<Option<bool>>,
    pub time_till_power_up: Option<Vec<Option<i32>>>,
    pub rigid_body: RigidBodyFrames,
    pub throttle: Vec<Option<u8>>,
    pub steer: Vec<Option<u8>>,
    pub handbrake: Vec<Option<bool>>,
    pub primary_color: Option<u8>,
    pub accent_color: Option<u8>,
    pub primary_finish: u32,
    pub accent_finish: u32,
    pub camera_settings: Option<CamSettings>,
    pub jump_active: Vec<u8>,
    pub double_jump_active: Vec<u8>,
    pub dodge_active: Vec<u8>,
    pub boost_active: Vec<bool>,
    pub boost: Vec<Option<f32>>,
    pub boost_collect: Vec<bool>,
    pub loadout: Loadouts,
    pub loadout_paints: LoadoutPaints,
    pub loadout_user_colors: LoadoutUserColors,
    pub power_up_active: Vec<Option<bool>>,
    pub power_up: Vec<Option<String>>,
    pub rumble_item_events: Vec<RumbleItemEvent>,
}

impl PlayerData {
    pub fn with_capacity(c: usize) -> Self {
        let mut data = PlayerData {
            remote_id: None,
            name: None,
            is_bot: false,
            team_actor: -1,
            match_score: 0,
            goals: 0,
            assists: 0,
            saves: 0,
            shots: 0,
            party_leader: None,
            title: None,
            total_xp: None,
            steering_sensitivity: None,
            ping: Vec::with_capacity(c),
            ball_cam: Vec::with_capacity(c),
            time_till_power_up: None,
            rigid_body: RigidBodyFrames::with_capacity(c),
            throttle: Vec::with_capacity(c),
            steer: Vec::with_capacity(c),
            handbrake: Vec::with_capacity(c),
            jump_active: Vec::with_capacity(c),
            double_jump_active: Vec::with_capacity(c),
            dodge_active: Vec::with_capacity(c),
            boost_active: Vec::with_capacity(c),
            boost: Vec::with_capacity(c),
            boost_collect: Vec::with_capacity(c),
            power_up_active: Vec::with_capacity(c),
            power_up: Vec::with_capacity(c),
            loadout: Loadouts::new(),
            loadout_paints: LoadoutPaints::new(),
            loadout_user_colors: LoadoutUserColors::new(),
            primary_color: None,
            accent_color: None,
            primary_finish: 270,
            accent_finish: 270,
            camera_settings: None,
            rumble_item_events: Vec::new(),
        };

        data.ping.resize(c, None);
        data.ball_cam.resize(c, None);
        data.throttle.resize(c, None);
        data.steer.resize(c, None);
        data.handbrake.resize(c, None);
        data.jump_active.resize(c, 0);
        data.double_jump_active.resize(c, 0);
        data.dodge_active.resize(c, 0);
        data.boost_active.resize(c, false);
        data.boost.resize(c, None);
        data.boost_collect.resize(c, false);
        data.power_up_active.resize(c, None);
        data.power_up.resize(c, None);

        data
    }

    pub fn new_frame(&mut self, frame: usize, delta: f32) {
        if frame > 0 {
            self.ping[frame] = self.ping[frame - 1].clone();
            self.ball_cam[frame] = self.ball_cam[frame - 1].clone();
            self.throttle[frame] = self.throttle[frame - 1].clone();
            self.steer[frame] = self.steer[frame - 1].clone();
            self.handbrake[frame] = self.handbrake[frame - 1].clone();
            self.jump_active[frame] = self.jump_active[frame - 1].clone();
            self.double_jump_active[frame] = self.double_jump_active[frame - 1].clone();
            self.dodge_active[frame] = self.dodge_active[frame - 1].clone();
            self.boost_active[frame] = self.boost_active[frame - 1].clone();
            self.power_up_active[frame] = self.power_up_active[frame - 1].clone();
            self.power_up[frame] = self.power_up[frame - 1].clone();

            if self.boost_active[frame - 1] {
                self.boost[frame] = Some((self.boost[frame - 1].unwrap_or(0.0) -
                    delta * BOOST_PER_SECOND).max(0.0));
            } else {
                self.boost[frame] = self.boost[frame - 1].clone();
            }

            self.time_till_power_up.as_mut().map(|arr| arr[frame] = arr[frame - 1].clone());
        }
        self.rigid_body.new_frame(frame);
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Demolition {
    pub attacker_player_id: i32,
    pub victim_player_id: i32,
    pub attack_velocity: Vector3f,
    pub victim_velocity: Vector3f,
    pub frame_number: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Loadouts {
    pub blue: Loadout,
    pub orange: Option<Loadout>,
}

impl Loadouts {
    pub fn new() -> Self {
        Loadouts {
            blue: Loadout {
                version: 0,
                body: 23,
                decal: 0,
                wheels: 376,
                rocket_trail: 0,
                antenna: 0,
                topper: 0,
                unknown1: 0,
                unknown2: None,
                engine_audio: None,
                trail: None,
                goal_explosion: None,
                banner: None,
                unknown3: None,
            },
            orange: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LoadoutPaints {
    pub blue: Paints,
    pub orange: Option<Paints>,
}

impl LoadoutPaints {
    pub fn new() -> Self {
        LoadoutPaints {
            blue: Paints::new(),
            orange: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Paints {
    pub body: Option<u32>,
    pub decal: Option<u32>,
    pub wheels: Option<u32>,
    pub rocket_trail: Option<u32>,
    pub antenna: Option<u32>,
    pub topper: Option<u32>,
    pub trail: Option<u32>,
    pub goal_explosion: Option<u32>,
    pub banner: Option<u32>,
    pub avatar_border: Option<u32>,
}

impl Paints {
    pub fn new() -> Self {
        Paints {
            body: None,
            decal: None,
            wheels: None,
            rocket_trail: None,
            antenna: None,
            topper: None,
            trail: None,
            goal_explosion: None,
            banner: None,
            avatar_border: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LoadoutUserColors {
    pub blue: UserColors,
    pub orange: Option<UserColors>,
}

impl LoadoutUserColors {
    pub fn new() -> Self {
        LoadoutUserColors {
            blue: UserColors::new(),
            orange: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct UserColors {
    pub banner: Option<u32>,
    pub avatar_border: Option<u32>,
}

impl UserColors {
    pub fn new() -> Self {
        UserColors {
            banner: None,
            avatar_border: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DropshotData {
    pub tile_frames: HashMap<u32, Vec<u8>>,
    pub damage_events: HashMap<usize, DropshotDamageEvent>,
    pub ball_events: Vec<DropshotBallEvent>,
}

impl DropshotData {
    pub fn new() -> Self {
        DropshotData {
            tile_frames: HashMap::new(),
            damage_events: HashMap::new(),
            ball_events: Vec::new(),
        }
    }

    pub fn new_frame(&mut self, frame: usize) {
        for (_, tile_frames) in &mut self.tile_frames {
            tile_frames[frame] = tile_frames[frame - 1];
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DropshotDamageEvent {
    pub offender: i32,
    pub tiles: Vec<DropshotTile>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DropshotTile {
    pub tile_id: u32,
    pub state: u8,
    pub direct_hit: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DropshotBallEvent {
    pub state: u8,
    pub frame_number: usize,
    pub team: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RumbleItemEvent {
    pub frame_get: usize,
    pub frame_use: Option<usize>,
    pub item_name: String,
    #[serde(skip_serializing)]
    pub demoed: bool,
}
