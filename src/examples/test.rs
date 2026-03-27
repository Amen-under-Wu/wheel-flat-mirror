use crate::{cartridge::CartContext, system::SystemContext, wrapper::InternalProgram};
use std::{cell::RefCell, rc::Rc};
pub struct TestCart {
    cart: Rc<RefCell<CartContext>>,
    system: Rc<RefCell<SystemContext>>,
    t: i32,
}
impl TestCart {
    pub fn new() -> Self {
        Self {
            cart: Rc::new(RefCell::new(CartContext::new())),
            system: Rc::new(RefCell::new(SystemContext::new())),
            t: 0,
        }
    }
}
impl InternalProgram for TestCart {
    fn init(&mut self, cart: Rc<RefCell<CartContext>>, system: Rc<RefCell<SystemContext>>) {
        self.cart = cart.clone();
        self.system = system.clone();
        let mut cart = self.cart.borrow_mut();
        for i in 0..16 {
            //cart.poke4(0xffe4*2+i, 0);
            //cart.poke4(0xffe4*2+16+i, 15);
            cart.poke4(0xffe4 * 2 + i, i as u8);
            cart.poke4(0xffe4 * 2 + 16 + i, 15 - i as u8);
        }
        //const ARP: [u8; 3] = [0,4,7];
        for i in 0..30 {
            cart.poke(0x100e4 + i * 2, (i + 1) as u8 / 2);
            //cart.poke(0x100e4+i*2+1, ARP[i%3]);
        }
    }
    fn update(&mut self) {
        let mut cart = self.cart.borrow_mut();
        cart.cls(0);
        let vols: Vec<u8> = (0..3)
            .map(|i| cart.peek(0xff9c + i * 18 + 1) >> 4)
            .collect();
        cart.print(
            &format!("{},{},{}", vols[0], vols[1], vols[2]),
            0,
            0,
            13,
            true,
            1,
            true,
        );

        if self.t % 120 == 0 {
            cart.sfx(0, 36, -1, 0, 15, 0);
        }
        if self.t % 120 == 10 {
            cart.sfx(0, 36 + 4, -1, 1, 15, 0);
        }
        if self.t % 120 == 20 {
            cart.sfx(0, 36 + 7, -1, 2, 15, 0);
        }
        if cart.key(Some(66)) {
            self.system.borrow_mut().exit();
        }
        self.t += 1;
    }
    fn scanline(&mut self, _i: usize) {}
    fn overlay(&mut self) {}
    fn to_file(&self) -> Option<Vec<u8>> {
        None
    }
}
