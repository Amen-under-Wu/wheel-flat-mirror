mod cartridge;
mod data;
mod io_device;
mod script;
mod system;
mod web_bindings;
mod wheel_file;
mod wrapper;

use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::WebGl2RenderingContext as GL;

use crate::script::WheelScript;
use crate::script::js::JsScript;

struct WheelContext {
    screen: Box<dyn io_device::Display>,
    speaker: Box<dyn io_device::PlayRegister>,
    file_io: Box<dyn io_device::FileIO>,
    vbuffer: Vec<u8>,
    abuffer: [io_device::WheelSoundRegister; 4],
    ibuffer: Box<dyn io_device::GetInput>,
    fbuffer: Vec<u8>,
    file_flag: bool,
}

impl WheelContext {
    fn new() -> Self {
        let window = web_sys::window().expect("no global window exists");
        let document = window.document().expect("no document on window");
        let canvas = document
            .get_element_by_id("canvas")
            .expect("canvas element not found")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap();
        let ratio = 800 / (240 + 16);
        canvas.set_width((240 + 16) * ratio);
        canvas.set_height((136 + 8) * ratio);
        let gl = canvas
            .get_context("webgl2")
            .unwrap()
            .unwrap()
            .dyn_into::<GL>()
            .unwrap();
        let mut screen = web_bindings::Screen::new(gl);
        screen.adjust_size(canvas.width() as f32);

        let rect = canvas.get_bounding_client_rect();

        let ibuffer = web_bindings::InputDevice::new_refcell();
        web_bindings::InputDevice::link(
            &ibuffer,
            &canvas.dyn_into::<web_sys::EventTarget>().unwrap(),
        );
        ibuffer.borrow_mut().set_rect(
            rect.x() as i32,
            rect.y() as i32,
            rect.width() as i32,
            rect.height() as i32,
            web_bindings::Screen::WIDTH as i32,
            web_bindings::Screen::HEIGHT as i32,
        );

        Self {
            screen: Box::new(screen),
            speaker: Box::new(web_bindings::DummySpeaker::new()),
            file_io: Box::new(web_bindings::FileDevice::new()),
            vbuffer: vec![
                0;
                (web_bindings::Screen::WIDTH * web_bindings::Screen::HEIGHT * 3) as usize
            ],
            abuffer: [io_device::WheelSoundRegister::new(); 4],
            ibuffer: Box::new(ibuffer),
            fbuffer: Vec::new(),
            file_flag: false,
        }
    }
    fn update(&mut self) {
        self.screen.display_screen(&self.vbuffer);
        self.speaker.set_registers(&self.abuffer);
        if self.file_flag {
            if let Some(file) = self.file_io.read_file() {
                self.fbuffer = file;
                self.file_flag = false;
            }
        }
    }
    fn in_screen(x: i32, y: i32) -> bool {
        x >= 0
            && y >= 0
            && (x as u32) < web_bindings::Screen::WIDTH
            && (y as u32) < web_bindings::Screen::HEIGHT
    }
}

pub trait WheelInterface {
    fn draw_pixel(&mut self, x: i32, y: i32, color: u32);
    fn draw_pixel_unsafe(&mut self, x: i32, y: i32, color: u32);
    fn play(&mut self, channel: usize, waveform: [u8; 32], volumn: u8, freq: u16);
    fn get_buttons(&self) -> [u8; 4];
    fn get_keys(&self) -> [u8; 4];
    fn get_mouse(&self) -> io_device::MouseData;
    fn upload_file(&mut self);
    fn read_file(&mut self) -> Vec<u8>;
    fn save_file(&mut self, name: &str, data: Vec<u8>);
}

impl WheelInterface for WheelContext {
    fn draw_pixel(&mut self, x: i32, y: i32, color: u32) {
        if Self::in_screen(x, y) {
            let idx: usize = (y as u32 * web_bindings::Screen::WIDTH + x as u32) as usize * 3;
            self.vbuffer[idx] = ((color >> 16) & 0xff) as u8;
            self.vbuffer[idx + 1] = ((color >> 8) & 0xff) as u8;
            self.vbuffer[idx + 2] = (color & 0xff) as u8;
        }
    }
    fn draw_pixel_unsafe(&mut self, x: i32, y: i32, color: u32) {
        let idx: usize = (y as u32 * web_bindings::Screen::WIDTH + x as u32) as usize * 3;
        self.vbuffer[idx] = ((color >> 16) & 0xff) as u8;
        self.vbuffer[idx + 1] = ((color >> 8) & 0xff) as u8;
        self.vbuffer[idx + 2] = (color & 0xff) as u8;
    }
    fn play(&mut self, channel: usize, waveform: [u8; 32], volumn: u8, freq: u16) {
        if channel < 4 {
            self.abuffer[channel].waveform = waveform;
            self.abuffer[channel].volumn = volumn;
            self.abuffer[channel].freq = freq;
        }
    }
    fn get_buttons(&self) -> [u8; 4] {
        self.ibuffer.get_input().gamepad
    }
    fn get_keys(&self) -> [u8; 4] {
        self.ibuffer.get_input().key
    }
    fn get_mouse(&self) -> io_device::MouseData {
        self.ibuffer.get_input().mouse
    }
    fn upload_file(&mut self) {
        self.file_io.upload_file();
        self.file_flag = true;
    }
    fn read_file(&mut self) -> Vec<u8> {
        self.fbuffer.clone()
    }
    fn save_file(&mut self, name: &str, data: Vec<u8>) {
        self.file_io.write_file(name, &data);
    }
}

pub trait WheelProgram {
    fn init(&mut self, wheel: &mut dyn WheelInterface) {}
    fn update(&mut self, wheel: &mut dyn WheelInterface) {}
}

#[wasm_bindgen]
struct Wheel {
    context: WheelContext,
    program: Box<dyn WheelProgram>,
}

#[wasm_bindgen]
impl Wheel {
    pub fn new() -> Self {
        let context = WheelContext::new();
        let mut program = Box::new(wrapper::WheelWrapper::new());
        //program.programs.insert("demo_0".to_string(), Rc::new(RefCell::new(DemoProgram::new())));
        let mut js_script = JsScript::new();
        let script_str = include_str!("demo.js");
        js_script.load(script_str).unwrap();
        program
            .programs
            .insert("demo".to_string(), Rc::new(RefCell::new(js_script)));
        Self { context, program }
    }
    pub fn update(&mut self) {
        self.program.update(&mut self.context);
        self.context.update();
    }
}
