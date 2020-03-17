use crate::attributes::RigidBody;
use std::f32::consts::PI;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ParsedFrameData {
    pub frames_data: FramesData,
    pub ball_data: BallData,
    pub player_data: HashMap<i32, PlayerData>,
}

impl ParsedFrameData {
    pub fn with_capacity(c: usize) -> Self {
        ParsedFrameData {
            frames_data: FramesData::with_capacity(c),
            ball_data: BallData::with_capacity(c),
            player_data: HashMap::new(),
        }
    }

    pub fn new_frame(&mut self, time: f32, delta: f32) {
        self.frames_data.new_frame(time, delta);
        self.ball_data.new_frame();
        for (_, data) in &mut self.player_data {
            data.new_frame();
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
}

impl FramesData {
    pub fn with_capacity(c: usize) -> Self {
        FramesData {
            time: Vec::with_capacity(c),
            delta: Vec::with_capacity(c),
            seconds_remaining: Vec::with_capacity(c),
            replicated_seconds_remaining: Vec::with_capacity(c),
            is_overtime: Vec::with_capacity(c),
            ball_has_been_hit: Vec::with_capacity(c),
        }
    }

    pub fn new_frame(&mut self, time: f32, delta: f32) {
        self.time.push(time);
        self.delta.push(delta);
        self.seconds_remaining.push(self.seconds_remaining.last().unwrap_or(&None).clone());
        self.replicated_seconds_remaining.push(self.replicated_seconds_remaining.last().unwrap_or(&None).clone());
        self.is_overtime.push(self.is_overtime.last().unwrap_or(&None).clone());
        self.ball_has_been_hit.push(self.ball_has_been_hit.last().unwrap_or(&None).clone());
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
        RigidBodyFrames {
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
        }
    }

    pub fn add_rigid_body(&mut self, i: usize, rb: &RigidBody) {
        self.pos_x[i] = Some(rb.location.x);
        self.pos_y[i] = Some(rb.location.y);
        self.pos_z[i] = Some(rb.location.z);

        // convert quat to euler
        let sinr = 2.0 * (rb.rotation.w * rb.rotation.x + rb.rotation.y * rb.rotation.z);
        let cosr = 1.0 - 2.0 * (rb.rotation.x * rb.rotation.x + rb.rotation.y * rb.rotation.y);
        let roll = sinr.atan2(cosr);

        let sinp = 2.0 * (rb.rotation.w * rb.rotation.y - rb.rotation.z * rb.rotation.x);

        let pitch = if sinp.abs() > 1.0 {
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

        match rb.linear_velocity {
            Some(v) => {
                self.vel_x[i] = Some(v.x);
                self.vel_y[i] = Some(v.y);
                self.vel_z[i] = Some(v.z);
            }
            None => {}
        }

        match rb.angular_velocity {
            Some(v) => {
                self.ang_vel_x[i] = Some(v.x);
                self.ang_vel_y[i] = Some(v.y);
                self.ang_vel_z[i] = Some(v.z);
            }
            None => {}
        }
    }

    pub fn new_frame(&mut self) {
        self.pos_x.push(self.pos_x.last().unwrap_or(&None).clone());
        self.pos_y.push(self.pos_y.last().unwrap_or(&None).clone());
        self.pos_z.push(self.pos_z.last().unwrap_or(&None).clone());
        self.rot_x.push(self.rot_x.last().unwrap_or(&None).clone());
        self.rot_y.push(self.rot_y.last().unwrap_or(&None).clone());
        self.rot_z.push(self.rot_z.last().unwrap_or(&None).clone());
        self.vel_x.push(self.vel_x.last().unwrap_or(&None).clone());
        self.vel_y.push(self.vel_y.last().unwrap_or(&None).clone());
        self.vel_z.push(self.vel_z.last().unwrap_or(&None).clone());
        self.ang_vel_x.push(self.ang_vel_x.last().unwrap_or(&None).clone());
        self.ang_vel_y.push(self.ang_vel_y.last().unwrap_or(&None).clone());
        self.ang_vel_z.push(self.ang_vel_z.last().unwrap_or(&None).clone());
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
    pub ball_type: BallType,
    pub rigid_body: RigidBodyFrames,
    pub hit_team_no: Vec<Option<u8>>,
}

impl BallData {
    pub fn with_capacity(c: usize) -> Self {
        BallData {
            ball_type: BallType::Unknown,
            rigid_body: RigidBodyFrames::with_capacity(c),
            hit_team_no: Vec::with_capacity(c),
        }
    }

    pub fn new_frame(&mut self) {
        self.rigid_body.new_frame();
        self.hit_team_no.push(self.hit_team_no.last().unwrap_or(&None).clone());
    }
}


#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PlayerData {
    pub name: Option<String>,
    pub team_actor: i32,
    pub ping: Vec<Option<u8>>,
    pub ball_cam: Vec<Option<bool>>,
    pub time_till_power_up: Option<Vec<Option<i32>>>,
}

impl PlayerData {
    pub fn with_capacity(c: usize) -> Self {
        PlayerData {
            name: None,
            team_actor: -1,
            ping: Vec::with_capacity(c),
            ball_cam: Vec::with_capacity(c),
            time_till_power_up: None,
        }
    }

    pub fn new_frame(&mut self) {
        self.ping.push(self.ping.last().unwrap_or(&None).clone());
        self.ball_cam.push(self.ball_cam.last().unwrap_or(&None).clone());
        match &mut self.time_till_power_up {
            Some(arr) => arr.push(arr.last().unwrap_or(&None).clone()),
            None => {}
        }
    }
}
