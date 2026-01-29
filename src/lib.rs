mod io_device;
use crate::io_device::{graphics_device, audio_device};
use web_sys::WebGl2RenderingContext as GL;
use wasm_bindgen::prelude::*;
use rand::Rng;


#[wasm_bindgen]
pub struct Wheel {
    screen: Box<dyn graphics_device::Display>,
    speaker: audio_device::Speaker,
    buffer: Vec<u8>,
    rng: rand::rngs::ThreadRng,
    t: i32
}

#[wasm_bindgen]
impl Wheel {
    pub fn new(audio_context: web_sys::AudioContext) -> Self {
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

        Self {
            screen: Box::new(screen),
            speaker: audio_device::Speaker::new(audio_context),
            buffer: vec![0; (graphics_device::Screen::WIDTH * graphics_device::Screen::HEIGHT * 3) as usize],
            rng: rand::thread_rng(),
            t: 0
        }
    }
    pub fn update(&mut self) {
        let x = self.rng.gen_range(0..graphics_device::Screen::WIDTH as usize);
        let y = self.rng.gen_range(0..graphics_device::Screen::HEIGHT as usize);
        let r = self.rng.r#gen::<u8>();
        let g = self.rng.r#gen::<u8>();
        let b = self.rng.r#gen::<u8>();
        for i in 0..graphics_device::Screen::WIDTH as usize {
            self.buffer[(y * graphics_device::Screen::WIDTH as usize + i) * 3] = r;
            self.buffer[(y * graphics_device::Screen::WIDTH as usize + i) * 3 + 1] = g;
            self.buffer[(y * graphics_device::Screen::WIDTH as usize + i) * 3 + 2] = b;
        }
        for i in 0..graphics_device::Screen::HEIGHT as usize {
            self.buffer[(i * graphics_device::Screen::WIDTH as usize + x) * 3] = r;
            self.buffer[(i * graphics_device::Screen::WIDTH as usize + x) * 3 + 1] = g;
            self.buffer[(i * graphics_device::Screen::WIDTH as usize + x) * 3 + 2] = b;
        }
        self.screen.display_screen(&self.buffer);
        if self.t % 60 == 0 {
            self.speaker.start();
        } else if self.t % 60 == 30 {
            self.speaker.stop();
        }
        self.t += 1;
    }
}
