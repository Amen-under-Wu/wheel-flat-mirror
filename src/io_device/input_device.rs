use wasm_bindgen::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::{HashSet, HashMap};

#[derive(Clone, Debug)]
pub struct MouseData {
    pub left: bool,
    pub right: bool,
    pub middle: bool,
    pub x: i32,
    pub y: i32,
    pub scroll_x: i32,
    pub scroll_y: i32,
}
impl MouseData {
    fn new() -> Self {
        Self {
            left: false,
            right: false,
            middle: false,
            x: 0,
            y: 0,
            scroll_x: 0,
            scroll_y: 0,
        }
    }
}

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
        Self {x, y, w, h, pix_w, pix_h}
    }
    pub fn project(&self, x: i32, y: i32) -> (i32, i32) {
        ((x - self.x) * self.pix_w / self.w, (y - self.y) * self.pix_h / self.h)
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
            "KeyA", "KeyB", "KeyC", "KeyD", "KeyE", "KeyF", "KeyG", "KeyH", "KeyI", "KeyJ", "KeyK", "KeyL", "KeyM", "KeyN", "KeyO", "KeyP", "KeyQ", "KeyR", "KeyS", "KeyT", "KeyU", "KeyV", "KeyW", "KeyX", "KeyY", "KeyZ",
            "Digit0", "Digit1", "Digit2", "Digit3", "Digit4", "Digit5", "Digit6", "Digit7", "Digit8", "Digit9",
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
            "F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8", "F9", "F10", "F11", "F12",
            "Numpad0", "Numpad1", "Numpad2", "Numpad3", "Numpad4", "Numpad5", "Numpad6", "Numpad7", "Numpad8", "Numpad9",
            "NumpadAdd", "NumpadSubtract", "NumpadMultiply", "NumpadDivide", "NumpadEnter", "NumpadDecimal",
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
        let closure_move = wasm_bindgen::closure::Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            let mut inner = self_clone.borrow_mut();
            let coord = inner.screen_rect.project(event.page_x(), event.page_y());
            // maybe better with event.offset_x()
            (inner.mouse.x, inner.mouse.y) = coord;
        }) as Box<dyn FnMut(_)>);
        target.add_event_listener_with_callback(
            "mousemove",
            closure_move.as_ref().unchecked_ref()
        ).unwrap();
        closure_move.forget();

        let self_clone = Rc::clone(_self);
        let closure_mousedown = wasm_bindgen::closure::Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            let mut inner = self_clone.borrow_mut();
            match event.button() {
                0 => {inner.mouse.left = true;}
                1 => {inner.mouse.middle = true;}
                2 => {inner.mouse.right = true;}
                _ => ()
            }
        }) as Box<dyn FnMut(_)>);
        target.add_event_listener_with_callback(
            "mousedown",
            closure_mousedown.as_ref().unchecked_ref()
        ).unwrap();
        closure_mousedown.forget();

        let self_clone = Rc::clone(_self);
        let closure_mouseup = wasm_bindgen::closure::Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            let mut inner = self_clone.borrow_mut();
            match event.button() {
                0 => {inner.mouse.left = false;}
                1 => {inner.mouse.middle = false;}
                2 => {inner.mouse.right = false;}
                _ => ()
            }
            event.prevent_default();
        }) as Box<dyn FnMut(_)>);
        target.add_event_listener_with_callback(
            "mouseup",
            closure_mouseup.as_ref().unchecked_ref()
        ).unwrap();
        closure_mouseup.forget();

        let self_clone = Rc::clone(_self);
        let closure_wheel = wasm_bindgen::closure::Closure::wrap(Box::new(move |event: web_sys::WheelEvent| {
            let mut inner = self_clone.borrow_mut();
            inner.mouse.scroll_x = event.delta_x() as i32;
            inner.mouse.scroll_y = event.delta_y() as i32;
            event.prevent_default();
        }) as Box<dyn FnMut(_)>);
        target.add_event_listener_with_callback(
            "wheel",
            closure_wheel.as_ref().unchecked_ref()
        ).unwrap();
        closure_wheel.forget();

        let self_clone = Rc::clone(_self);
        let closure_keydown = wasm_bindgen::closure::Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
            let mut inner = self_clone.borrow_mut();
            let code = event.code();
            event.prevent_default();
            inner.key_buffer.insert(code);
        }) as Box<dyn FnMut(_)>);
        target.add_event_listener_with_callback(
            "keydown",
            closure_keydown.as_ref().unchecked_ref()
        ).unwrap();
        closure_keydown.forget();

        let self_clone = Rc::clone(_self);
        let closure_keyup = wasm_bindgen::closure::Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
            let mut inner = self_clone.borrow_mut();
            inner.key_buffer.remove(&event.code());
        }) as Box<dyn FnMut(_)>);
        target.add_event_listener_with_callback(
            "keyup",
            closure_keyup.as_ref().unchecked_ref()
        ).unwrap();
        closure_keyup.forget();

        let closure_contextmenu = wasm_bindgen::closure::Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            // Only prevent default if it's a right click (button 2)
            if event.button() == 2 {
                event.prevent_default();
                event.stop_propagation();
            }
        }) as Box<dyn FnMut(_)>);
        target.add_event_listener_with_callback(
            "contextmenu",
            closure_contextmenu.as_ref().unchecked_ref()
        ).unwrap();
        closure_contextmenu.forget();
    }
    pub fn set_rect(&mut self, x: i32, y: i32, w: i32, h: i32, pix_w: i32, pix_h: i32) {
        self.screen_rect = ScreenRect::new(x, y, w, h, pix_w, pix_h);
    }
}

pub struct WheelInputBuffer {
    pub gamepad: [u8; Self::GAMEPAD_BUFFER_SIZE],
    pub mouse: MouseData,
    pub key: [u8; Self::KEY_BUFFER_SIZE],
}

impl WheelInputBuffer {
    const GAMEPAD_BUFFER_SIZE: usize = 4;
    const KEY_BUFFER_SIZE: usize = 4;
}

pub trait GetInput {
    fn get_input(&self) -> WheelInputBuffer;
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
