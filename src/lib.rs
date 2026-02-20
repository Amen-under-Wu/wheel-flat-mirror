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

struct Program {
    _rng: rand::rngs::ThreadRng,
    _t: u32,
    click_pos: (i32, i32),
    shape: i32,
}

impl Program {
    fn new() -> Self {
        Self {
            _rng: rand::thread_rng(),
            _t: 0,
            click_pos: (0, 0),
            shape: 0,
        }
    }
}

impl cartridge::CartProgram for Program {
    fn init(&mut self, context: &mut cartridge::CartContext) {
        for i in 0..240 {
            for j in 0..136 {
                if i % 8 == 0 {
                    context.set_pix(i, j, 1);
                }
                if j % 8 == 0 {
                    context.set_pix(i, j, 1);
                }
            }
        }
    }
    fn update(&mut self, context: &mut cartridge::CartContext) {
        context.cls(0);
        let (x, y, left, _, _, _, _) = context.mouse();
        let x: i32 = x.into();
        let y: i32 = y.into();
        if left {
            if self.click_pos == (-1, -1) {
                self.click_pos = (x, y);
            }
            match self.shape {
                0 => { context.rect(self.click_pos.0, self.click_pos.1, (x - self.click_pos.0).abs() + 1, (y - self.click_pos.1).abs() + 1, 1); }, 
                1 => { context.rectb(self.click_pos.0, self.click_pos.1, (x - self.click_pos.0).abs() + 1, (y - self.click_pos.1).abs() + 1, 1); }, 
                2 => { context.circ(self.click_pos.0, self.click_pos.1, ((x - self.click_pos.0).abs() + 1).max((y - self.click_pos.1).abs() + 1), 1); }
                3 => { context.circb(self.click_pos.0, self.click_pos.1, ((x - self.click_pos.0).abs() + 1).max((y - self.click_pos.1).abs() + 1), 1); }
                4 => { context.elli(self.click_pos.0, self.click_pos.1, (x - self.click_pos.0).abs() + 1, (y - self.click_pos.1).abs() + 1, 1); }, 
                5 => { context.ellib(self.click_pos.0, self.click_pos.1, (x - self.click_pos.0).abs() + 1, (y - self.click_pos.1).abs() + 1, 1); }, 
                6 => { context.line(self.click_pos.0 as f32, self.click_pos.1 as f32, x as f32, y as f32, 1); }, 
                _ => (),
            }
        } else {
            self.click_pos = (-1, -1);
        }
        if context.btnp_with_hold_period(4, 60, 10) || context.keyp_with_hold_period(2, 60, 10) {
            let color1 = (context.peek4(0x3ff0 * 2 + 1) + 1) % 16;
            context.poke4(0x3ff0 * 2 + 1, color1);
        }
        if context.btnp(5) {
            self.shape += 1;
            self.shape %= 7;
        }
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
