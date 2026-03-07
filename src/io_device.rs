pub trait Display {
    fn display_screen(&mut self, screen_buffer: &Vec<u8>);
    fn resize(&mut self, w: u32);
}

#[derive(Copy, Clone)]
pub struct WheelSoundRegister {
    pub waveform: [u8; Self::WAVEFORM_LENGTH],
    pub volumn: u8,
    pub freq: u16,
}

impl WheelSoundRegister {
    pub const WAVEFORM_LENGTH: usize = 32;
    pub fn new() -> Self {
        Self {
            waveform: [0; Self::WAVEFORM_LENGTH],
            volumn: 0,
            freq: 0,
        }
    }
}

pub trait PlayRegister {
    fn set_registers(&mut self, reg: &[WheelSoundRegister]);
}

#[derive(Clone, Debug)]
pub struct MouseData {
    pub left: bool,
    pub right: bool,
    pub middle: bool,
    pub x: i32,
    pub y: i32,
    pub scroll_x: i32,
    pub scroll_y: i32,
}
impl MouseData {
    pub fn new() -> Self {
        Self {
            left: false,
            right: false,
            middle: false,
            x: 0,
            y: 0,
            scroll_x: 0,
            scroll_y: 0,
        }
    }
}

pub struct WheelInputBuffer {
    pub gamepad: [u8; Self::GAMEPAD_BUFFER_SIZE],
    pub mouse: MouseData,
    pub key: [u8; Self::KEY_BUFFER_SIZE],
}

impl WheelInputBuffer {
    pub const GAMEPAD_BUFFER_SIZE: usize = 4;
    pub const KEY_BUFFER_SIZE: usize = 4;
}

pub trait GetInput {
    fn get_input(&self) -> WheelInputBuffer;
}

pub trait FileIO {
    fn upload_file(&self);
    fn read_file(&self) -> Option<Vec<u8>>;
    fn write_file(&self, path: &str, data: &[u8]) -> bool;
}
