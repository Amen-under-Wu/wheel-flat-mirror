use wasm_bindgen::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;

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
pub struct InputDevice {
    pub key_buffer: [u8; Self::KEY_BUFFER_SIZE],
    pub mouse: MouseData
}

impl InputDevice {
    const KEY_BUFFER_SIZE: usize = 4;
    pub fn new() -> Self {
        let res = Self {
            key_buffer: [0; Self::KEY_BUFFER_SIZE],
            mouse: MouseData::new(),
        };
        res
    }
    pub fn new_refcell() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self::new()))
    }
    pub fn link(_self: &Rc<RefCell<Self>>, target: &web_sys::EventTarget) {
        let self_clone = Rc::clone(_self);
        let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            let mut inner = self_clone.borrow_mut();
            inner.mouse.x = event.client_x();
            inner.mouse.y = event.client_y();
        }) as Box<dyn FnMut(_)>);

        target.add_event_listener_with_callback(
            "click",
            closure.as_ref().unchecked_ref()
        ).unwrap();
        closure.forget();
    }
}
