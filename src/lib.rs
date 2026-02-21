mod io_device;
mod cartridge;
mod data;
use crate::io_device::{graphics_device, audio_device, input_device};
use web_sys::WebGl2RenderingContext as GL;
use wasm_bindgen::prelude::*;
use rand::Rng;

struct WheelContext {
    screen: Box<dyn graphics_device::Display>,
    speaker: Box<dyn audio_device::PlayRegister>,
    vbuffer: Vec<u8>,
    abuffer: [audio_device::WheelSoundRegister; 4],
    ibuffer: Box<dyn input_device::GetInput>,
}

impl WheelContext {
    fn new(audio_context: web_sys::AudioContext) -> Self {
        let window = web_sys::window().expect("no global window exists");
        let document = window.document().expect("no document on window");
        let canvas = document
            .get_element_by_id("canvas")
            .expect("canvas element not found")
            .dyn_into::<web_sys::HtmlCanvasElement>().unwrap();
        canvas.set_width((240+16)*4);
        canvas.set_height((136+8)*4);
        let gl = canvas.get_context("webgl2").unwrap().unwrap().dyn_into::<GL>().unwrap();
        let mut screen = graphics_device::Screen::new(gl);
        screen.adjust_size(canvas.width() as f32);

        let rect = canvas.get_bounding_client_rect();

        let ibuffer = input_device::InputDevice::new_refcell();
        input_device::InputDevice::link(&ibuffer, &canvas.dyn_into::<web_sys::EventTarget>().unwrap());
        ibuffer.borrow_mut().set_rect(rect.x() as i32, rect.y() as i32, rect.width() as i32, rect.height() as i32,
            graphics_device::Screen::WIDTH as i32, graphics_device::Screen::HEIGHT as i32);

        Self {
            screen: Box::new(screen),
            speaker: Box::new(audio_device::Speaker::new(audio_context)),
            vbuffer: vec![0; (graphics_device::Screen::WIDTH * graphics_device::Screen::HEIGHT * 3) as usize],
            abuffer: [audio_device::WheelSoundRegister::new(); 4],
            ibuffer: Box::new(ibuffer),
        }
    }
    fn update(&mut self) {
        self.screen.display_screen(&self.vbuffer);
        self.speaker.set_registers(&self.abuffer);
    }
    fn in_screen(x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && (x as u32) < graphics_device::Screen::WIDTH && (y as u32) < graphics_device::Screen::HEIGHT
    }
}

pub trait WheelInterface {
    fn draw_pixel(&mut self, x: i32, y: i32, color: u32);
    fn draw_pixel_unsafe(&mut self, x: i32, y: i32, color: u32);
    fn play(&mut self, channel: usize, waveform: [u8; 32], volumn: u8, freq: u16);
    fn get_buttons(&self) -> [u8; 4];
    fn get_keys(&self) -> [u8; 4];
    fn get_mouse(&self) -> input_device::MouseData;
}

impl WheelInterface for WheelContext {
    fn draw_pixel(&mut self, x: i32, y: i32, color: u32) {
        if Self::in_screen(x, y) {
            let idx: usize = (y as u32 * graphics_device::Screen::WIDTH + x as u32) as usize * 3;
            self.vbuffer[idx] = ((color >> 16) & 0xff) as u8;
            self.vbuffer[idx + 1] = ((color >> 8) & 0xff) as u8;
            self.vbuffer[idx + 2] = (color & 0xff) as u8;
        }
    }
    fn draw_pixel_unsafe(&mut self, x: i32, y: i32, color: u32) {
        let idx: usize = (y as u32 * graphics_device::Screen::WIDTH + x as u32) as usize * 3;
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
    fn get_mouse(&self) -> input_device::MouseData {
        self.ibuffer.get_input().mouse
    }
}

pub trait WheelProgram {
    fn init(&mut self, wheel: &mut dyn WheelInterface) {}
    fn update(&mut self, wheel: &mut dyn WheelInterface) {}
}

use std::collections::HashMap;

struct Program {
    rng: rand::rngs::ThreadRng,
    i32_data: HashMap<String, i32>,
}

impl Program {
    fn new() -> Self {
        Self {
            rng: rand::thread_rng(),
            i32_data: HashMap::new(),
        }
    }
}

impl cartridge::CartProgram for Program {
    fn init(&mut self, context: &mut cartridge::CartContext) {
        self.i32_data.insert("t".to_string(), 0);
        self.i32_data.insert("x".to_string(), 0);
        self.i32_data.insert("y".to_string(), 0);
        self.i32_data.insert("sx".to_string(), 96);
        self.i32_data.insert("sy".to_string(), 24);
        self.i32_data.insert("shape".to_string(), 0);
        self.i32_data.insert("color".to_string(), 1);
        context.poke(0x4000, 0x22);
        context.poke(0x8000, 1);
    }
    fn update(&mut self, context: &mut cartridge::CartContext) {
        context.cls(13);
        context.map(1, 1, 10, 10, 0, 0, 255, 1);
        context.print("Hello World!", 84, 84, 0, false, 2, false);
        if context.btn(0) {
            web_sys::console::log_1(&"up".into());
            self.i32_data.entry("sy".to_string()).and_modify(|y| *y -= 1).or_insert(24);
        }
        if context.btn(1) {
            self.i32_data.entry("sy".to_string()).and_modify(|y| *y += 1).or_insert(24);
        }
        if context.btn(2) {
            self.i32_data.entry("sx".to_string()).and_modify(|x| *x -= 1).or_insert(96);
        }
        if context.btn(3) {
            self.i32_data.entry("sx".to_string()).and_modify(|x| *x += 1).or_insert(96);
        }
        context.spr(1+self.i32_data["t"]%60/30*2,self.i32_data["sx"],self.i32_data["sy"],14,3,0,0,2,2);
        let (x, y, left, _, _, _, _) = context.mouse();
        let x: i32 = x.into();
        let y: i32 = y.into();
        if left {
            if self.i32_data["x"] == -1 {
                *self.i32_data.get_mut("x").unwrap() = x;
                *self.i32_data.get_mut("y").unwrap() = y;
            }
            let x0 = self.i32_data["x"];
            let y0 = self.i32_data["y"];
            let color = self.i32_data["color"] as u8;
            match self.i32_data["shape"] {
                0 => { context.rect(x0, y0, (x - x0).abs() + 1, (y - y0).abs() + 1, color); }, 
                1 => { context.rectb(x0, y0, (x - x0).abs() + 1, (y - y0).abs() + 1, color); }, 
                2 => { context.circ(x0, y0, ((x - x0).abs() + 1).max((y - y0).abs() + 1), color); }
                3 => { context.circb(x0, y0, ((x - x0).abs() + 1).max((y - y0).abs() + 1), color); }
                4 => { context.elli(x0, y0, (x - x0).abs() + 1, (y - y0).abs() + 1, color); }, 
                5 => { context.ellib(x0, y0, (x - x0).abs() + 1, (y - y0).abs() + 1, color); }, 
                6 => { context.line(x0 as f32, y0 as f32, x as f32, y as f32, color); }, 
                _ => (),
            }
        } else {
            *self.i32_data.get_mut("x").unwrap() = -1;
            *self.i32_data.get_mut("y").unwrap() = -1;
        }
        if context.btnp_with_hold_period(4, 60, 10) || context.keyp_with_hold_period(2, 60, 10) {
            self.i32_data.entry("color".to_string()).and_modify(|x| *x = (*x + 1) % 16).or_insert(0);
        }
        if context.btnp(5) {
            self.i32_data.entry("shape".to_string()).and_modify(|x| *x = (*x + 1) % 7).or_insert(0);
        }

        self.i32_data.entry("t".to_string()).and_modify(|x| *x += 1).or_insert(0);
    }
}


#[wasm_bindgen]
struct Wheel {
    context: WheelContext,
    program: cartridge::Cartridge,
}

#[wasm_bindgen]
impl Wheel {
    pub fn new(audio_context: web_sys::AudioContext) -> Self {
        let mut context = WheelContext::new(audio_context);
        let mut program = cartridge::Cartridge::new(Box::new(Program::new()));
        program.init(&mut context);
        Self {
            context,
            program,
        }
    }
    pub fn update(&mut self) {
        self.program.update(&mut self.context);
        self.context.update();
    }
}
