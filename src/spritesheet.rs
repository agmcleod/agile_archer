extern crate serde;

#[derive(Serialize, Deserialize)]
pub struct FrameData {
    x: usize,
    y: usize,
    w: usize,
    h: usize,
}

#[derive(Serialize, Deserialize)]
pub struct Size {
    w: usize,
    h: usize,
}

#[derive(Serialize, Deserialize)]
pub struct Frame {
    filename: String,
    frame: FrameData,
    rotated: bool,
    trimmed: bool,
    spriteSourceSize: FrameData,
    sourceSize: Size,
}

#[derive(Serialize, Deserialize)]
pub struct Spritesheet {
    frames: Vec<Frame>,
}