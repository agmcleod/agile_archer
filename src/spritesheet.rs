extern crate serde;

#[derive(Serialize, Deserialize)]
pub struct FrameData {
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
}

#[derive(Serialize, Deserialize)]
pub struct Size {
    pub w: usize,
    pub h: usize,
}

#[derive(Serialize, Deserialize)]
pub struct Frame {
    pub filename: String,
    pub frame: FrameData,
    pub rotated: bool,
    pub trimmed: bool,
    pub spriteSourceSize: FrameData,
    pub sourceSize: Size,
}

#[derive(Serialize, Deserialize)]
pub struct Meta {
    pub size: Size,
}

#[derive(Serialize, Deserialize)]
pub struct Spritesheet {
    pub frames: Vec<Frame>,
    pub meta: Meta
}