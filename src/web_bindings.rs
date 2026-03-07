use crate::io_device::{self, *};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::WebGl2RenderingContext as GL;

pub struct Screen {
    gl: GL,
    program: web_sys::WebGlProgram,
    buffer: Vec<f32>,
    canvas_width: f32,
}
impl Screen {
    pub const WIDTH: u32 = (240 + 16) * 2;
    pub const HEIGHT: u32 = (136 + 8) * 2;
    const VERTEX_SHADER_SOURCE: &str = r#"
attribute vec2 a_position;
attribute vec3 a_color;

// 传递给片元着色器的变量
varying vec3 v_color;

uniform float u_pixelSize;

void main() {
gl_Position = vec4(a_position.x, -a_position.y, 0.0, 1.0);

gl_PointSize = u_pixelSize;

v_color = a_color;
}
    "#;
    const FRAGMENT_SHADER_SOURCE: &str = r#"
precision mediump float;

varying vec3 v_color;

void main() {
gl_FragColor = vec4(v_color, 1.0);
}
    "#;
    pub fn new(gl: GL) -> Self {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        let vertex_shader =
            Self::compile_shader(&gl, GL::VERTEX_SHADER, Self::VERTEX_SHADER_SOURCE);
        let fragment_shader =
            Self::compile_shader(&gl, GL::FRAGMENT_SHADER, Self::FRAGMENT_SHADER_SOURCE);
        let program = Self::create_program(&gl, &vertex_shader, &fragment_shader).unwrap();
        gl.use_program(Some(&program));
        let buffer = gl.create_buffer().unwrap();
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&buffer));

        let pos_attr_loc = gl.get_attrib_location(&program, "a_position") as u32;
        let color_attr_loc = gl.get_attrib_location(&program, "a_color") as u32;
        gl.enable_vertex_attrib_array(pos_attr_loc);
        gl.enable_vertex_attrib_array(color_attr_loc);
        let stride = 5 * 4;
        gl.vertex_attrib_pointer_with_i32(pos_attr_loc, 2, GL::FLOAT, false, stride, 0);
        gl.vertex_attrib_pointer_with_i32(color_attr_loc, 3, GL::FLOAT, false, stride, 2 * 4);

        let mut buffer = Vec::<f32>::new();
        buffer.reserve((5 * Self::WIDTH * Self::HEIGHT) as usize);
        for i in 0..Self::HEIGHT as usize {
            for j in 0..Self::WIDTH as usize {
                //buffer[(i * Self::WIDTH as usize + j) * 5] = (j * 2 + 1) as f32 / Self::WIDTH as f32 - 1.0;
                //buffer[(i * Self::WIDTH as usize + j) * 5 + 1] = (i * 2 + 1) as f32 / Self::HEIGHT as f32 - 1.0;
                buffer.push((j * 2 + 1) as f32 / Self::WIDTH as f32 - 1.0);
                buffer.push((i * 2 + 1) as f32 / Self::HEIGHT as f32 - 1.0);
                buffer.push(0.0);
                buffer.push(0.0);
                buffer.push(0.0);
            }
        }

        Self {
            gl,
            program,
            buffer,
            canvas_width: 0.0,
        }
    }
    fn compile_shader(gl: &GL, _type: u32, source: &str) -> web_sys::WebGlShader {
        let shader = gl.create_shader(_type).expect("failed to create shader");
        gl.shader_source(&shader, source);
        gl.compile_shader(&shader);
        shader
    }
    fn create_program(
        gl: &GL,
        vertex_shader: &web_sys::WebGlShader,
        fragment_shader: &web_sys::WebGlShader,
    ) -> Option<web_sys::WebGlProgram> {
        let program = gl.create_program().expect("failed to create program");
        gl.attach_shader(&program, vertex_shader);
        gl.attach_shader(&program, fragment_shader);
        gl.link_program(&program);
        if !(gl
            .get_program_parameter(&program, GL::LINK_STATUS)
            .as_bool()
            .unwrap_or(false))
        {
            gl.delete_program(Some(&program));
            None
        } else {
            Some(program)
        }
    }
    pub fn adjust_size(&mut self, canvas_w: f32) {
        let u_pix_size = self
            .gl
            .get_uniform_location(&self.program, "u_pixelSize")
            .unwrap();
        let pixel_size = canvas_w / Self::WIDTH as f32;
        self.gl.uniform1f(Some(&u_pix_size), pixel_size);
        let canvas_h = canvas_w as u32 * Self::HEIGHT / Self::WIDTH;
        self.gl.viewport(0, 0, canvas_w as i32, canvas_h as i32);
        self.canvas_width = canvas_w;
    }
    pub fn update(&mut self, buffer: &Vec<u8>) {
        for i in 0..(Self::WIDTH * Self::HEIGHT) as usize {
            for j in 0..3 {
                self.buffer[i * 5 + 2 + j] = buffer[i * 3 + j] as f32 / 255.0;
            }
        }
    }
    pub fn display(&self) {
        let vertices_array = {
            let memory_buffer = wasm_bindgen::memory()
                .dyn_into::<js_sys::WebAssembly::Memory>()
                .unwrap()
                .buffer();
            let location: u32 = self.buffer.as_ptr() as u32 / 4;
            js_sys::Float32Array::new(&memory_buffer)
                .subarray(location, location + self.buffer.len() as u32)
        };
        self.gl.buffer_data_with_array_buffer_view(
            GL::ARRAY_BUFFER,
            &vertices_array,
            GL::STATIC_DRAW,
        );
        self.gl
            .draw_arrays(GL::POINTS, 0, (Self::WIDTH * Self::HEIGHT) as i32);
    }
}
impl Display for Screen {
    fn display_screen(&mut self, screen_buffer: &Vec<u8>) {
        self.update(screen_buffer);
        self.display();
    }
    fn resize(&mut self, w: u32) {
        self.adjust_size(w as f32);
    }
}

pub struct DummySpeaker {}
impl DummySpeaker {
    pub fn new() -> Self {
        Self {}
    }
}
impl PlayRegister for DummySpeaker {
    fn set_registers(&mut self, _reg: &[WheelSoundRegister]) {
        // do nothing
    }
}

//use wasm_bindgen::prelude::*;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

pub struct ScreenRect {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    pix_w: i32,
    pix_h: i32,
}
impl ScreenRect {
    pub fn new(x: i32, y: i32, w: i32, h: i32, pix_w: i32, pix_h: i32) -> Self {
        Self {
            x,
            y,
            w,
            h,
            pix_w,
            pix_h,
        }
    }
    pub fn project(&self, x: i32, y: i32) -> (i32, i32) {
        (
            (x - self.x) * self.pix_w / self.w,
            (y - self.y) * self.pix_h / self.h,
        )
    }
}

pub struct InputDevice {
    pub key_buffer: HashSet<String>,
    pub mouse: MouseData,
    pub screen_rect: ScreenRect,
    key_map: HashMap<String, u8>,
}

impl InputDevice {
    pub fn new() -> Self {
        let key_vec = vec![
            "",
            "KeyA",
            "KeyB",
            "KeyC",
            "KeyD",
            "KeyE",
            "KeyF",
            "KeyG",
            "KeyH",
            "KeyI",
            "KeyJ",
            "KeyK",
            "KeyL",
            "KeyM",
            "KeyN",
            "KeyO",
            "KeyP",
            "KeyQ",
            "KeyR",
            "KeyS",
            "KeyT",
            "KeyU",
            "KeyV",
            "KeyW",
            "KeyX",
            "KeyY",
            "KeyZ",
            "Digit0",
            "Digit1",
            "Digit2",
            "Digit3",
            "Digit4",
            "Digit5",
            "Digit6",
            "Digit7",
            "Digit8",
            "Digit9",
            "Minus",
            "Equal",
            "BracketLeft",
            "BracketRight",
            "Backslash",
            "Semicolon",
            "Quote",
            "Backquote",
            "Comma",
            "Period",
            "Slash",
            "Space",
            "Tab",
            "Enter",
            "Backspace",
            "Delete",
            "Insert",
            "PageUp",
            "PageDown",
            "Home",
            "End",
            "ArrowUp",
            "ArrowDown",
            "ArrowLeft",
            "ArrowRight",
            "CapsLock",
            "ControlLeft",
            "ShiftLeft",
            "AltLeft",
            "Escape",
            "F1",
            "F2",
            "F3",
            "F4",
            "F5",
            "F6",
            "F7",
            "F8",
            "F9",
            "F10",
            "F11",
            "F12",
            "Numpad0",
            "Numpad1",
            "Numpad2",
            "Numpad3",
            "Numpad4",
            "Numpad5",
            "Numpad6",
            "Numpad7",
            "Numpad8",
            "Numpad9",
            "NumpadAdd",
            "NumpadSubtract",
            "NumpadMultiply",
            "NumpadDivide",
            "NumpadEnter",
            "NumpadDecimal",
        ];
        let mut key_map = HashMap::new();
        for i in 1..key_vec.len() {
            key_map.insert(key_vec[i].to_string(), i as u8);
        }
        let res = Self {
            key_buffer: HashSet::new(),
            mouse: MouseData::new(),
            screen_rect: ScreenRect::new(0, 0, 0, 0, 0, 0),
            key_map,
        };
        res
    }
    fn key_code(&self, key: &str) -> u8 {
        *self.key_map.get(key).unwrap_or(&0)
    }
    pub fn new_refcell() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self::new()))
    }
    pub fn link(_self: &Rc<RefCell<Self>>, target: &web_sys::EventTarget) {
        let self_clone = Rc::clone(_self);
        let closure_move =
            wasm_bindgen::closure::Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
                let mut inner = self_clone.borrow_mut();
                let coord = inner.screen_rect.project(event.page_x(), event.page_y());
                // maybe better with event.offset_x()
                (inner.mouse.x, inner.mouse.y) = coord;
            }) as Box<dyn FnMut(_)>);
        target
            .add_event_listener_with_callback("mousemove", closure_move.as_ref().unchecked_ref())
            .unwrap();
        closure_move.forget();

        let self_clone = Rc::clone(_self);
        let closure_mousedown =
            wasm_bindgen::closure::Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
                let mut inner = self_clone.borrow_mut();
                match event.button() {
                    0 => {
                        inner.mouse.left = true;
                    }
                    1 => {
                        inner.mouse.middle = true;
                    }
                    2 => {
                        inner.mouse.right = true;
                    }
                    _ => (),
                }
            }) as Box<dyn FnMut(_)>);
        target
            .add_event_listener_with_callback(
                "mousedown",
                closure_mousedown.as_ref().unchecked_ref(),
            )
            .unwrap();
        closure_mousedown.forget();

        let self_clone = Rc::clone(_self);
        let closure_mouseup =
            wasm_bindgen::closure::Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
                let mut inner = self_clone.borrow_mut();
                match event.button() {
                    0 => {
                        inner.mouse.left = false;
                    }
                    1 => {
                        inner.mouse.middle = false;
                    }
                    2 => {
                        inner.mouse.right = false;
                    }
                    _ => (),
                }
                event.prevent_default();
            }) as Box<dyn FnMut(_)>);
        target
            .add_event_listener_with_callback("mouseup", closure_mouseup.as_ref().unchecked_ref())
            .unwrap();
        closure_mouseup.forget();

        let self_clone = Rc::clone(_self);
        let closure_wheel =
            wasm_bindgen::closure::Closure::wrap(Box::new(move |event: web_sys::WheelEvent| {
                let mut inner = self_clone.borrow_mut();
                inner.mouse.scroll_x = event.delta_x() as i32;
                inner.mouse.scroll_y = event.delta_y() as i32;
                event.prevent_default();
            }) as Box<dyn FnMut(_)>);
        target
            .add_event_listener_with_callback("wheel", closure_wheel.as_ref().unchecked_ref())
            .unwrap();
        closure_wheel.forget();

        let self_clone = Rc::clone(_self);
        let closure_keydown =
            wasm_bindgen::closure::Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
                let mut inner = self_clone.borrow_mut();
                let code = event.code();
                event.prevent_default();
                inner.key_buffer.insert(code);
            }) as Box<dyn FnMut(_)>);
        target
            .add_event_listener_with_callback("keydown", closure_keydown.as_ref().unchecked_ref())
            .unwrap();
        closure_keydown.forget();

        let self_clone = Rc::clone(_self);
        let closure_keyup =
            wasm_bindgen::closure::Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
                let mut inner = self_clone.borrow_mut();
                inner.key_buffer.remove(&event.code());
            }) as Box<dyn FnMut(_)>);
        target
            .add_event_listener_with_callback("keyup", closure_keyup.as_ref().unchecked_ref())
            .unwrap();
        closure_keyup.forget();

        let closure_contextmenu =
            wasm_bindgen::closure::Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
                // Only prevent default if it's a right click (button 2)
                if event.button() == 2 {
                    event.prevent_default();
                    event.stop_propagation();
                }
            }) as Box<dyn FnMut(_)>);
        target
            .add_event_listener_with_callback(
                "contextmenu",
                closure_contextmenu.as_ref().unchecked_ref(),
            )
            .unwrap();
        closure_contextmenu.forget();
    }
    pub fn set_rect(&mut self, x: i32, y: i32, w: i32, h: i32, pix_w: i32, pix_h: i32) {
        self.screen_rect = ScreenRect::new(x, y, w, h, pix_w, pix_h);
    }
}

impl GetInput for Rc<RefCell<InputDevice>> {
    fn get_input(&self) -> WheelInputBuffer {
        let mut res = WheelInputBuffer {
            gamepad: [0; WheelInputBuffer::GAMEPAD_BUFFER_SIZE],
            mouse: self.borrow().mouse.clone(),
            key: [0; WheelInputBuffer::KEY_BUFFER_SIZE],
        };
        let mut key_code_vec = Vec::<u8>::new();
        for key in &self.borrow().key_buffer {
            let code = self.borrow().key_code(key);
            if code != 0 {
                key_code_vec.push(code);
            }
        }
        key_code_vec.sort();
        for i in 0..key_code_vec.len().min(WheelInputBuffer::KEY_BUFFER_SIZE) {
            res.key[i] = key_code_vec[i];
        }
        res
    }
}

pub struct FileDevice {}

impl FileDevice {
    pub fn new() -> Self {
        Self {}
    }
}

impl io_device::FileIO for FileDevice {
    fn upload_file(&self) {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let input = document
            .query_selector("input")
            .unwrap()
            .unwrap_or(document.create_element("input").unwrap())
            .dyn_into::<web_sys::HtmlInputElement>()
            .unwrap();
        input.set_type("file");
        input.click();
    }
    fn read_file(&self) -> Option<Vec<u8>> {
        let window = web_sys::window()?;

        // 获取window.getFileData函数
        let get_file_data =
            js_sys::Reflect::get(&window, &JsValue::from_str("getFileData")).ok()?;

        // 检查是否是函数
        if !get_file_data.is_function() {
            return None;
        }

        // 调用函数获取Uint8Array
        let file_data_js = js_sys::Function::from(get_file_data).call0(&window).ok()?;

        // 检查是否为空
        if file_data_js.is_null() || file_data_js.is_undefined() {
            return None;
        }
        let uint8array = file_data_js.dyn_into::<js_sys::Uint8Array>().ok()?;

        // 转换为Uint8Array
        let uint8array = js_sys::Uint8Array::from(uint8array);

        // 创建Vec<u8>并复制数据
        let mut vec = vec![0; uint8array.length() as usize];
        uint8array.copy_to(&mut vec);

        Some(vec)
    }
    fn write_file(&self, path: &str, data: &[u8]) -> bool {
        let blob = web_sys::Blob::new_with_u8_array_sequence(&js_sys::Array::of1(
            &js_sys::Uint8Array::from(data),
        ))
        .unwrap();
        let url = web_sys::Url::create_object_url_with_blob(&blob).unwrap();
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let a = document
            .create_element("a")
            .unwrap()
            .dyn_into::<web_sys::HtmlAnchorElement>()
            .unwrap();
        a.set_href(&url);
        a.set_download(path);
        a.click();
        web_sys::Url::revoke_object_url(&url).unwrap();
        true
    }
}
