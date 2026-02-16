mod io_device;
mod cartridge;
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
    fn draw_while(&mut self, from: i32, to: i32, color: u32, coord: &dyn Fn(i32) -> (i32, i32));
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
    fn draw_while(&mut self, from: i32, to: i32, color: u32, coord: &dyn Fn(i32) -> (i32, i32)) {
        for i in from..to {
            let (x, y) = coord(i);
            if !Self::in_screen(x, y) {
                break;
            }
            let idx: usize = (y as u32 * graphics_device::Screen::WIDTH + x as u32) as usize * 3;
            self.vbuffer[idx] = ((color >> 16) & 0xff) as u8;
            self.vbuffer[idx + 1] = ((color >> 8) & 0xff) as u8;
            self.vbuffer[idx + 2] = (color & 0xff) as u8;
        }
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

struct Program {
    rng: rand::rngs::ThreadRng,
    t: u32,
}

impl Program {
    fn new() -> Self {
        Self {
            rng: rand::thread_rng(),
            t: 0
        }
    }
    fn init(&mut self, wheel: &mut dyn WheelInterface) {
        for i in 0..graphics_device::Screen::WIDTH as i32 {
            for j in 0..graphics_device::Screen::HEIGHT as i32 {
                wheel.draw_pixel(i, j, 0xffffff);
            }
        }
    }
    fn update(&mut self, wheel: &mut dyn WheelInterface) {
        let mouse = wheel.get_mouse();
        let x = mouse.x;
        let y = mouse.y;
        if mouse.left {
            let rgb = self.rng.r#gen::<u32>();
            for i in 0..graphics_device::Screen::WIDTH as i32 {
                wheel.draw_pixel(i, y, rgb);
            }
            for i in 0..graphics_device::Screen::HEIGHT as i32 {
                wheel.draw_pixel(x, i, rgb);
            }
        } else if mouse.right {
            for i in 0..graphics_device::Screen::WIDTH as i32 {
                wheel.draw_pixel(i, y, 0xffffff);
            }
            for i in 0..graphics_device::Screen::HEIGHT as i32 {
                wheel.draw_pixel(x, i, 0xffffff);
            }
        }
        let volumn = 15;
        let freq = 440;
        let mut waveform = [0; 32];
        if self.t % 60 == 0 {
            for i in 0..32 {
                waveform[i] = if i < 16 {0} else {15};
            }
        } else if self.t % 60 == 30 {
            for i in 0..32 {
                waveform[i] = i as u8 / 2;
            }
        }
        wheel.play(0, waveform, volumn, freq);
        self.t += 1;
    }
}

#[wasm_bindgen]
struct Wheel {
    context: WheelContext,
    program: Program,
}

#[wasm_bindgen]
impl Wheel {
    pub fn new(audio_context: web_sys::AudioContext) -> Self {
        let mut context = WheelContext::new(audio_context);
        let mut program = Program::new();
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
