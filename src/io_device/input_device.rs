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
    pub key_buffer: [u8; Self::KEY_BUFFER_SIZE],
    pub mouse: MouseData,
    pub screen_rect: ScreenRect,
}

impl InputDevice {
    const KEY_BUFFER_SIZE: usize = 4;
    pub fn new() -> Self {
        let res = Self {
            key_buffer: [0; Self::KEY_BUFFER_SIZE],
            mouse: MouseData::new(),
            screen_rect: ScreenRect::new(0, 0, 0, 0, 0, 0),
        };
        res
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
            event.prevent_default();
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
