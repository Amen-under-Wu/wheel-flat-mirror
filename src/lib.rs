mod io_device;
use crate::io_device::io_device as my_io;
use web_sys::WebGl2RenderingContext as GL;
use wasm_bindgen::prelude::*;
use rand::Rng;


#[wasm_bindgen]
pub struct Wheel {
    screen: my_io::Screen,
    buffer: Vec<u8>,
    rng: rand::rngs::ThreadRng
}

#[wasm_bindgen]
impl Wheel {
    pub fn new() -> Self {
        let window = web_sys::window().expect("no global window exists");
        let document = window.document().expect("no document on window");
        let canvas = document
            .get_element_by_id("canvas")
            .expect("canvas element not found")
            .dyn_into::<web_sys::HtmlCanvasElement>().unwrap();
        canvas.set_width((240+16)*4);
        canvas.set_height((136+8)*4);
        let gl = canvas.get_context("webgl2").unwrap().unwrap().dyn_into::<GL>().unwrap();
        let mut screen = my_io::Screen::new(gl);
        screen.adjust_size(canvas.width() as f32);
        Self {
            screen,
            buffer: vec![0; (my_io::Screen::WIDTH * my_io::Screen::HEIGHT * 3) as usize],
            rng: rand::thread_rng()
        }
    }
    pub fn update(&mut self) {
        let x = self.rng.gen_range(0..my_io::Screen::WIDTH as usize);
        let y = self.rng.gen_range(0..my_io::Screen::HEIGHT as usize);
        let r = self.rng.r#gen::<u8>();
        let g = self.rng.r#gen::<u8>();
        let b = self.rng.r#gen::<u8>();
        for i in 0..my_io::Screen::WIDTH as usize {
            self.buffer[(y * my_io::Screen::WIDTH as usize + i) * 3] = r;
            self.buffer[(y * my_io::Screen::WIDTH as usize + i) * 3 + 1] = g;
            self.buffer[(y * my_io::Screen::WIDTH as usize + i) * 3 + 2] = b;
        }
        for i in 0..my_io::Screen::HEIGHT as usize {
            self.buffer[(i * my_io::Screen::WIDTH as usize + x) * 3] = r;
            self.buffer[(i * my_io::Screen::WIDTH as usize + x) * 3 + 1] = g;
            self.buffer[(i * my_io::Screen::WIDTH as usize + x) * 3 + 2] = b;
        }
        self.screen.update(&self.buffer);
        self.screen.display();
    }
}
