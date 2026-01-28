mod io_device;
use crate::io_device::io_device as my_io;
use web_sys::WebGl2RenderingContext as GL;
use wasm_bindgen::prelude::*;


#[wasm_bindgen]
pub struct Wheel {
    screen: my_io::Screen
}

#[wasm_bindgen]
impl Wheel {
    pub fn new(gl: GL) -> Self {
        let mut screen = my_io::Screen::new(gl);
        screen.adjust_size(1200.0);
        Self {
            screen
        }
    }
    pub fn update(&mut self) {
        self.screen.update();
        self.screen.display();
    }
}
