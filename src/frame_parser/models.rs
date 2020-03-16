#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ParsedFrameData {
    pub frames_data: FramesData,
}

impl ParsedFrameData {
    pub fn with_capacity(c: usize) -> Self {
        ParsedFrameData {
            frames_data: FramesData::with_capacity(c),
        }
    }

    pub fn new_frame(&mut self, time: f32, delta: f32) {
        self.frames_data.new_frame(time, delta);
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
