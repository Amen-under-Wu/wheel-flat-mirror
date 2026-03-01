pub mod js;

use crate::{
    cartridge::CartContext,
    system::SystemContext,
};
use std::{
    rc::Rc,
    cell::RefCell,
};

pub trait WheelScript {
    fn bind(&mut self, cart: Rc<RefCell<CartContext>>, system: Rc<RefCell<SystemContext>>);
    fn load(&mut self, script: &str) -> Result<(), String>;
    fn update(&mut self) -> Result<(), String>;
    fn scanline(&mut self, line: i32) -> Result<(), String>;
    fn overlay(&mut self) -> Result<(), String>;
    fn log_error(&mut self, message: &str);
}

impl<T> crate::wrapper::InternalProgram for T
where
    T: WheelScript,
{
    fn init(&mut self, cart: Rc<RefCell<CartContext>>, system: Rc<RefCell<SystemContext>>) {
        self.bind(cart, system);
    }
    fn update(&mut self) {
        if let Err(e) = self.update() {
            self.log_error(&e);
        }
    }
    fn scanline(&mut self, i: usize) {
        if let Err(e) = self.scanline(i as i32) {
            self.log_error(&e);
        }
    }
    fn overlay(&mut self) {
        if let Err(e) = self.overlay() {
            self.log_error(&e);
        }
    }
}
