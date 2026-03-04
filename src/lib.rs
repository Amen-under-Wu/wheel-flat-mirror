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
        canvas.set_width((240 + 16) * 4);
        canvas.set_height((136 + 8) * 4);
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

/*
use std::collections::HashMap;

struct DemoProgram {
    cart: Option<Rc<RefCell<cartridge::CartContext>>>,
    sys: Option<Rc<RefCell<system::SystemContext>>>,
    i32_data: HashMap<String, i32>,
}

impl DemoProgram {
    fn new() -> Self {
        Self {
            cart: None,
            sys: None,
            i32_data: HashMap::new(),
        }
    }
}

impl wrapper::InternalProgram for DemoProgram {
    fn init(&mut self, cart: Rc<RefCell<cartridge::CartContext>>, system: Rc<RefCell<system::SystemContext>>) {
        cart.borrow_mut().poke(0x4000, 0x22);
        cart.borrow_mut().poke(0x8000, 1);
        system.borrow_mut().trace("运行demo", 13);
        self.cart = Some(cart);
        self.sys = Some(system);
        self.i32_data.insert("t".to_string(), 0);
        self.i32_data.insert("x".to_string(), 0);
        self.i32_data.insert("y".to_string(), 0);
        self.i32_data.insert("sx".to_string(), 96);
        self.i32_data.insert("sy".to_string(), 24);
        self.i32_data.insert("shape".to_string(), 0);
        self.i32_data.insert("color".to_string(), 1);
    }
    fn update(&mut self) {
        let binding = self.cart.clone().unwrap();
        let mut context = binding.borrow_mut();
        let binding = self.sys.clone().unwrap();
        let mut sys_context = binding.borrow_mut();
        context.cls(13);
        context.map(1, 1, 10, 10, 0, 0, 255, 1);
        context.print_ch("你好wheel flat轮扁!", 84, 84, 0, false, 1, false);
        context.print_ch("你好wheel flat轮扁!", 84, 94, 0, false, 1, true);
        context.print_ch("按esc回到终端", 84, 104, 0, false, 1, false);
        //context.print_ch("镧铈镨钕钷钐铕钆铽镝钬铒铥镱镥", 84, 104, 0, false, 1, true);
        if context.btn(0) {
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
                7 => { context.tri(0.0, 16.0, 16.0, 0.0, x as f32, y as f32, color); },
                8 => { context.trib(0.0, 16.0, 16.0, 0.0, x as f32, y as f32, color); },
                9 => { context.textri(0.0, 16.0, 16.0, 0.0, x as f32, y as f32, 0.0, 0.0, 32.0, 0.0, 0.0, 32.0, false, 0); },
                10 => { context.textri(0.0, 16.0, 16.0, 0.0, x as f32, y as f32, 0.0, 0.0, 32.0, 0.0, 0.0, 32.0, true, 0); },
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
            self.i32_data.entry("shape".to_string()).and_modify(|x| *x = (*x + 1) % 11).or_insert(0);
        }

        if context.keyp(Some(66)) {
            let t = sys_context.time();
            sys_context.trace(&format!("运行时间：{} ms", t), 13);
            sys_context.exit();
        }

        self.i32_data.entry("t".to_string()).and_modify(|x| *x += 1).or_insert(0);
    }
}
*/
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
    pub fn file_test(&mut self) {
        let data = self.context.read_file();
        web_sys::console::log_1(&format!("读取到文件，大小：{} bytes", data.len()).into());
    }
}
