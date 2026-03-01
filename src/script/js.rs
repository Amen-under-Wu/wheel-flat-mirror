use crate::script::WheelScript;
use wasm_bindgen::prelude::*;
use js_sys::{Function, Reflect};
use crate::{
    cartridge::CartContext,
    system::SystemContext,
};
use std::{
    rc::Rc,
    cell::RefCell,
};

pub struct JsScript {
    system: Option<Rc<RefCell<SystemContext>>>,
}

impl JsScript {
    pub fn new() -> Self {
        Self {
            system: None,
        }
    }
}

impl WheelScript for JsScript {
    fn bind(&mut self, cart: Rc<RefCell<CartContext>>, system: Rc<RefCell<SystemContext>>) {
        self.system = Some(system.clone());

        let global = js_sys::global();

        let cart_clone = cart.clone();
        let closure = Closure::wrap(Box::new(move || cart_clone.borrow().keyp(None)) as Box<dyn FnMut() -> bool>);
        let func = closure.as_ref().unchecked_ref::<Function>().clone();
        closure.forget();
        Reflect::set(&global, &"keyp_0".into(), &func).unwrap();

        let cart_clone = cart.clone();
        let closure = Closure::wrap(Box::new(move |i| cart_clone.borrow().keyp(Some(i))) as Box<dyn FnMut(u8) -> bool>);
        let func = closure.as_ref().unchecked_ref::<Function>().clone();
        closure.forget();
        Reflect::set(&global, &"keyp_1".into(), &func).unwrap();

        let cart_clone = cart.clone();
        let closure = Closure::wrap(Box::new(move |c| {cart_clone.borrow_mut().cls(c);}) as Box<dyn FnMut(u8)>);
        let func = closure.as_ref().unchecked_ref::<Function>().clone();
        closure.forget();
        Reflect::set(&global, &"cls_1".into(), &func).unwrap();

        let sys_clone = system.clone();
        let closure = Closure::wrap(Box::new(move |msg: JsValue, c| {sys_clone.borrow_mut().trace(msg.as_string().unwrap().as_str(), c);}) as Box<dyn FnMut(JsValue, u8)>);
        let func = closure.as_ref().unchecked_ref::<Function>().clone();
        closure.forget();
        Reflect::set(&global, &"trace".into(), &func).unwrap();

        let sys_clone = system.clone();
        let closure = Closure::wrap(Box::new(move || {sys_clone.borrow_mut().exit();}) as Box<dyn FnMut()>);
        let func = closure.as_ref().unchecked_ref::<Function>().clone();
        closure.forget();
        Reflect::set(&global, &"exit".into(), &func).unwrap();

    }
    fn load(&mut self, script: &str) -> Result<(), String> {
        let global = js_sys::global();
    
        let eval = Reflect::get(&global, &JsValue::from_str("eval")).map_err(|e| format!("Error accessing eval: {:?}", e))?
            .dyn_into::<Function>().map_err(|e| format!("Error converting eval to Function: {:?}", e))?;
        
        eval.call1(&JsValue::NULL, &JsValue::from_str(script)).map_err(|e| format!("Error executing script: {:?}", e))?;
        
        Ok(())
    }
    fn update(&mut self) -> Result<(), String> {
        js_sys::eval("update()").map_err(|e| format!("Error calling update: {:?}", e))?;
        Ok(())
    }
    fn scanline(&mut self, line: i32) -> Result<(), String> {
        js_sys::eval(&format!("if (typeof scanline !== 'undefined') {{ scanline({}); }}", line)).map_err(|e| format!("Error calling scanline: {:?}", e))?;
        Ok(())
    }
    fn overlay(&mut self) -> Result<(), String> {
        js_sys::eval("if (typeof overlay !== 'undefined') { overlay(); }").map_err(|e| format!("Error calling overlay: {:?}", e))?;
        Ok(())
    }
    fn log_error(&mut self, message: &str) {
        web_sys::console::error_1(&message.into());
        self.system.as_ref().unwrap().borrow_mut().trace(message, 2);
        self.system.as_ref().unwrap().borrow_mut().exit();
    }
}
