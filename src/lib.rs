mod io_device;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

struct FpsCounter {
    n: u32,
    timer: f64,
    performance: web_sys::Performance,
}
impl FpsCounter {
    pub fn new() -> Self {
        let window = web_sys::window().expect("should have a window in this context");
        let performance = window.performance().expect("performance should be available");
        Self {
            n: 0,
            timer: performance.now(),
            performance
        }
    }
    pub fn tick(&mut self) {
        let new_timer = self.performance.now();
        if new_timer - self.timer > 1000.0 {
            self.timer = new_timer;
            console_log!("{}", self.n);
            self.n = 0;
        }
        self.n += 1;
    }
}
