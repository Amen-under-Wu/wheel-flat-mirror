mod io_device;
use crate::io_device::{graphics_device, audio_device, input_device};
use web_sys::WebGl2RenderingContext as GL;
use wasm_bindgen::prelude::*;
use rand::Rng;
use std::rc::Rc;
use std::cell::RefCell;


#[wasm_bindgen]
pub struct Wheel {
    screen: Box<dyn graphics_device::Display>,
    speaker: Box<dyn audio_device::PlayRegister>,
    vbuffer: Vec<u8>,
    abuffer: [audio_device::WheelSoundRegister; 4],
    ibuffer: Rc<RefCell<input_device::InputDevice>>,
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
            ibuffer,
            rng: rand::thread_rng(),
            t: 0
        }
    }
    pub fn update(&mut self) {
        let x = self.ibuffer.borrow().mouse.x as usize % graphics_device::Screen::WIDTH as usize;
        let y = self.ibuffer.borrow().mouse.y as usize % graphics_device::Screen::HEIGHT as usize;
        if self.ibuffer.borrow().mouse.left {
            let r = self.rng.r#gen::<u8>();
            let g = self.rng.r#gen::<u8>();
            let b = self.rng.r#gen::<u8>();
            for i in 0..graphics_device::Screen::WIDTH as usize {
                self.vbuffer[(y * graphics_device::Screen::WIDTH as usize + i) * 3] = r;
                self.vbuffer[(y * graphics_device::Screen::WIDTH as usize + i) * 3 + 1] = g;
                self.vbuffer[(y * graphics_device::Screen::WIDTH as usize + i) * 3 + 2] = b;
            }
            for i in 0..graphics_device::Screen::HEIGHT as usize {
                self.vbuffer[(i * graphics_device::Screen::WIDTH as usize + x) * 3] = r;
                self.vbuffer[(i * graphics_device::Screen::WIDTH as usize + x) * 3 + 1] = g;
                self.vbuffer[(i * graphics_device::Screen::WIDTH as usize + x) * 3 + 2] = b;
            }
        } else if self.ibuffer.borrow().mouse.right {
            for i in 0..graphics_device::Screen::WIDTH as usize {
                self.vbuffer[(y * graphics_device::Screen::WIDTH as usize + i) * 3] = 0;
                self.vbuffer[(y * graphics_device::Screen::WIDTH as usize + i) * 3 + 1] = 0;
                self.vbuffer[(y * graphics_device::Screen::WIDTH as usize + i) * 3 + 2] = 0;
            }
            for i in 0..graphics_device::Screen::HEIGHT as usize {
                self.vbuffer[(i * graphics_device::Screen::WIDTH as usize + x) * 3] = 0;
                self.vbuffer[(i * graphics_device::Screen::WIDTH as usize + x) * 3 + 1] = 0;
                self.vbuffer[(i * graphics_device::Screen::WIDTH as usize + x) * 3 + 2] = 0;
            }
        }
        self.screen.display_screen(&self.vbuffer);
        self.abuffer[0].volumn = 15;
        self.abuffer[0].freq = 440;
        /*for i in 0..32 {
            self.abuffer[0].waveform[i] = if i < 16 {0} else {15};
        }
        if self.t % 60 == 0 {
            web_sys::console::log_1(
                &format!(
                    "mouse at x: {}, y: {}",
                    self.ibuffer.borrow().mouse.x,
                    self.ibuffer.borrow().mouse.y,
                )
                .into(),
            );
            //self.abuffer[0].freq = 440;
        } else if self.t % 60 == 30 {
            //self.abuffer[0].freq = 660;
        }*/
        if !self.ibuffer.borrow().key_buffer.is_empty() {
            web_sys::console::log_1(&format!("{:?}", self.ibuffer.borrow().key_buffer).into());
        }
        self.speaker.set_registers(&self.abuffer);
        self.t += 1;
    }
}
