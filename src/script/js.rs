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
    script: String,
    system: Option<Rc<RefCell<SystemContext>>>,
}

impl JsScript {
    pub fn new() -> Self {
        let global = js_sys::global();
        let eval = Reflect::get(&global, &JsValue::from_str("eval")).unwrap()
            .dyn_into::<Function>().unwrap();
        eval.call1(&JsValue::NULL, &JsValue::from_str(include_str!("js_prelude.js"))).unwrap();
        Self {
            script: String::new(),
            system: None,
        }
    }
}

impl WheelScript for JsScript {
    fn bind(&mut self, cart: Rc<RefCell<CartContext>>, system: Rc<RefCell<SystemContext>>) {
        self.system = Some(system.clone());

        let global = js_sys::global();

        let cart_clone = cart.clone();
        let closure = Closure::wrap(Box::new(move |i: i32| {
            i >= 0 && i < 32 && cart_clone.borrow().btn(i as u8)
        } ) as Box<dyn FnMut(i32) -> bool>);
        let func = closure.as_ref().unchecked_ref::<Function>().clone();
        closure.forget();
        Reflect::set(&global, &"btn".into(), &func).unwrap();

        let cart_clone = cart.clone();
        let closure = Closure::wrap(Box::new(move |i: i32| {
            i >= 0 && i < 32 && cart_clone.borrow().btnp(i as u8)
        }) as Box<dyn FnMut(i32) -> bool>);
        let func = closure.as_ref().unchecked_ref::<Function>().clone();
        closure.forget();
        Reflect::set(&global, &"btnp_1".into(), &func).unwrap();

        let cart_clone = cart.clone();
        let closure = Closure::wrap(Box::new(move |i: i32, h, p| {
            i >= 0 && i < 32 && cart_clone.borrow().btnp_with_hold_period(i as u8, h, p)
        }) as Box<dyn FnMut(i32, i32, i32) -> bool>);
        let func = closure.as_ref().unchecked_ref::<Function>().clone();
        closure.forget();
        Reflect::set(&global, &"btnp_3".into(), &func).unwrap();

        let cart_clone = cart.clone();
        let closure = Closure::wrap(Box::new(move |x, y, w, h| cart_clone.borrow_mut().clip(x, y, w, h)) as Box<dyn FnMut(i32, i32, i32, i32)>);
        let func = closure.as_ref().unchecked_ref::<Function>().clone();
        closure.forget();
        Reflect::set(&global, &"clip_4".into(), &func).unwrap();

        let cart_clone = cart.clone();
        let closure = Closure::wrap(Box::new(move |c: i32| {
            if c >= 0 && c < 16 {
                cart_clone.borrow_mut().cls(c as u8);
            }
        }) as Box<dyn FnMut(i32)>);
        let func = closure.as_ref().unchecked_ref::<Function>().clone();
        closure.forget();
        Reflect::set(&global, &"cls_1".into(), &func).unwrap();

        // circ, circb, elli, ... already have bound checks for color
        let cart_clone = cart.clone();
        let closure = Closure::wrap(Box::new(move |x, y, r, c: i32| cart_clone.borrow_mut().circ(x, y, r, c.try_into().unwrap_or(255))) as Box<dyn FnMut(i32, i32, i32, i32)>);
        let func = closure.as_ref().unchecked_ref::<Function>().clone();
        closure.forget();
        Reflect::set(&global, &"circ".into(), &func).unwrap();

        let cart_clone = cart.clone();
        let closure = Closure::wrap(Box::new(move |x, y, r, c: i32| cart_clone.borrow_mut().circb(x, y, r, c.try_into().unwrap_or(255))) as Box<dyn FnMut(i32, i32, i32, i32)>);
        let func = closure.as_ref().unchecked_ref::<Function>().clone();
        closure.forget();
        Reflect::set(&global, &"circb".into(), &func).unwrap();

        let sys_clone = system.clone();
        let closure = Closure::wrap(Box::new(move || {sys_clone.borrow_mut().exit();}) as Box<dyn FnMut()>);
        let func = closure.as_ref().unchecked_ref::<Function>().clone();
        closure.forget();
        Reflect::set(&global, &"exit".into(), &func).unwrap();

        let cart_clone = cart.clone();
        let closure = Closure::wrap(Box::new(move |x, y, a, b, c: i32| cart_clone.borrow_mut().elli(x, y, a, b, c.try_into().unwrap_or(255))) as Box<dyn FnMut(i32, i32, i32, i32, i32)>);
        let func = closure.as_ref().unchecked_ref::<Function>().clone();
        closure.forget();
        Reflect::set(&global, &"elli".into(), &func).unwrap();

        let cart_clone = cart.clone();
        let closure = Closure::wrap(Box::new(move |x, y, a, b, c: i32| cart_clone.borrow_mut().ellib(x, y, a, b, c.try_into().unwrap_or(255))) as Box<dyn FnMut(i32, i32, i32, i32, i32)>);
        let func = closure.as_ref().unchecked_ref::<Function>().clone();
        closure.forget();
        Reflect::set(&global, &"ellib".into(), &func).unwrap();

        let cart_clone = cart.clone();
        let closure = Closure::wrap(Box::new(move |i, f| cart_clone.borrow().fget(i, f)) as Box<dyn FnMut(i32, i32) -> bool>);
        let func = closure.as_ref().unchecked_ref::<Function>().clone();
        closure.forget();
        Reflect::set(&global, &"fget".into(), &func).unwrap();

        let cart_clone = cart.clone();
        let closure = Closure::wrap(Box::new(move |i, f, v| cart_clone.borrow_mut().fset(i, f, v)) as Box<dyn FnMut(i32, i32, bool)>);
        let func = closure.as_ref().unchecked_ref::<Function>().clone();
        closure.forget();
        Reflect::set(&global, &"fset".into(), &func).unwrap();

        // todo: font text, x, y, [transparent], [char width], [char height], [fixed=false], [scale=1] -> text width

        let cart_clone = cart.clone();
        let closure = Closure::wrap(Box::new(move || cart_clone.borrow().keyp(None)) as Box<dyn FnMut() -> bool>);
        let func = closure.as_ref().unchecked_ref::<Function>().clone();
        closure.forget();
        Reflect::set(&global, &"keyp_0".into(), &func).unwrap();

        let cart_clone = cart.clone();
        let closure = Closure::wrap(Box::new(move |i: i32| cart_clone.borrow().keyp(Some(i.try_into().unwrap_or(255)))) as Box<dyn FnMut(i32) -> bool>);
        let func = closure.as_ref().unchecked_ref::<Function>().clone();
        closure.forget();
        Reflect::set(&global, &"keyp_1".into(), &func).unwrap();

        let cart_clone = cart.clone();
        let closure = Closure::wrap(Box::new(move |i: i32, h, p| cart_clone.borrow().keyp_with_hold_period(i.try_into().unwrap_or(255), h, p)) as Box<dyn FnMut(i32, i32, i32) -> bool>);
        let func = closure.as_ref().unchecked_ref::<Function>().clone();
        closure.forget();
        Reflect::set(&global, &"keyp_3".into(), &func).unwrap();

        let cart_clone = cart.clone();
        let closure = Closure::wrap(Box::new(move |x1, y1, x2, y2, c: i32| cart_clone.borrow_mut().line(x1, y1, x2, y2, c.try_into().unwrap_or(255))) as Box<dyn FnMut(f32, f32, f32, f32, i32)>);
        let func = closure.as_ref().unchecked_ref::<Function>().clone();
        closure.forget();
        Reflect::set(&global, &"line".into(), &func).unwrap();

        let cart_clone = cart.clone();
        let closure = Closure::wrap(Box::new(move |x, y, w, h, sx, sy, c: i32, s| cart_clone.borrow_mut().map(x, y, w, h, sx, sy, c.try_into().unwrap_or(255), s)) as Box<dyn FnMut(i32, i32, i32, i32, i32, i32, i32, i32)>);
        let func = closure.as_ref().unchecked_ref::<Function>().clone();
        closure.forget();
        Reflect::set(&global, &"map_8".into(), &func).unwrap();

        let cart_clone = cart.clone();
        let closure = Closure::wrap(Box::new(move |t, f, l| cart_clone.borrow_mut().memcpy(t, f, l)) as Box<dyn FnMut(usize, usize, usize)>);
        let func = closure.as_ref().unchecked_ref::<Function>().clone();
        closure.forget();
        Reflect::set(&global, &"memcpy".into(), &func).unwrap();

        let cart_clone = cart.clone();
        let closure = Closure::wrap(Box::new(move |a, v, l| cart_clone.borrow_mut().memset(a, v, l)) as Box<dyn FnMut(usize, u8, usize)>);
        let func = closure.as_ref().unchecked_ref::<Function>().clone();
        closure.forget();
        Reflect::set(&global, &"memset".into(), &func).unwrap();

        let cart_clone = cart.clone();
        let closure = Closure::wrap(Box::new(move |x, y| cart_clone.borrow().mget(x, y)) as Box<dyn FnMut(i32, i32) -> i32>);
        let func = closure.as_ref().unchecked_ref::<Function>().clone();
        closure.forget();
        Reflect::set(&global, &"mget".into(), &func).unwrap();

        let cart_clone = cart.clone();
        let closure = Closure::wrap(Box::new(move |x, y, id| cart_clone.borrow_mut().mset(x, y, id)) as Box<dyn FnMut(i32, i32, u8)>);
        let func = closure.as_ref().unchecked_ref::<Function>().clone();
        closure.forget();
        Reflect::set(&global, &"mset".into(), &func).unwrap();

        let cart_clone = cart.clone();
        let closure = Closure::wrap(Box::new(move || {
            let mouse = cart_clone.borrow().mouse();
            let arr = js_sys::Array::new_with_length(7);
            arr.set(0, mouse.0.into());
            arr.set(1, mouse.1.into());
            arr.set(2, mouse.2.into());
            arr.set(3, mouse.3.into());
            arr.set(4, mouse.4.into());
            arr.set(5, mouse.5.into());
            arr.set(6, mouse.6.into());
            arr
        }) as Box<dyn FnMut() -> js_sys::Array>);
        let func = closure.as_ref().unchecked_ref::<Function>().clone();
        closure.forget();
        Reflect::set(&global, &"mouse".into(), &func).unwrap();

        // todo: music

        let cart_clone = cart.clone();
        let closure = Closure::wrap(Box::new(move |a| cart_clone.borrow().peek(a)) as Box<dyn FnMut(usize) -> u8>);
        let func = closure.as_ref().unchecked_ref::<Function>().clone();
        closure.forget();
        Reflect::set(&global, &"peek_1".into(), &func).unwrap();

        let cart_clone = cart.clone();
        let closure = Closure::wrap(Box::new(move |a, b| cart_clone.borrow().peek_with_bits(a, b)) as Box<dyn FnMut(usize, usize) -> u8>);
        let func = closure.as_ref().unchecked_ref::<Function>().clone();
        closure.forget();
        Reflect::set(&global, &"peek_2".into(), &func).unwrap();

        let cart_clone = cart.clone();
        let closure = Closure::wrap(Box::new(move |a| cart_clone.borrow().peek4(a)) as Box<dyn FnMut(usize) -> u8>);
        let func = closure.as_ref().unchecked_ref::<Function>().clone();
        closure.forget();
        Reflect::set(&global, &"peek4".into(), &func).unwrap();

        let cart_clone = cart.clone();
        let closure = Closure::wrap(Box::new(move |a| cart_clone.borrow().peek2(a)) as Box<dyn FnMut(usize) -> u8>);
        let func = closure.as_ref().unchecked_ref::<Function>().clone();
        closure.forget();
        Reflect::set(&global, &"peek2".into(), &func).unwrap();

        let cart_clone = cart.clone();
        let closure = Closure::wrap(Box::new(move |a| cart_clone.borrow().peek1(a)) as Box<dyn FnMut(usize) -> u8>);
        let func = closure.as_ref().unchecked_ref::<Function>().clone();
        closure.forget();
        Reflect::set(&global, &"peek1".into(), &func).unwrap();

        let cart_clone = cart.clone();
        let closure = Closure::wrap(Box::new(move |x, y| cart_clone.borrow().get_pix(x, y)) as Box<dyn FnMut(i32, i32) -> u8>);
        let func = closure.as_ref().unchecked_ref::<Function>().clone();
        closure.forget();
        Reflect::set(&global, &"pix_2".into(), &func).unwrap();

        let cart_clone = cart.clone();
        let closure = Closure::wrap(Box::new(move |x, y, c: i32| cart_clone.borrow_mut().set_pix(x, y, c.try_into().unwrap_or(255))) as Box<dyn FnMut(i32, i32, i32)>);
        let func = closure.as_ref().unchecked_ref::<Function>().clone();
        closure.forget();
        Reflect::set(&global, &"pix_3".into(), &func).unwrap();

        let cart_clone = cart.clone();
        let closure = Closure::wrap(Box::new(move |a| cart_clone.borrow().get_pmem(a)) as Box<dyn FnMut(usize) -> i32>);
        let func = closure.as_ref().unchecked_ref::<Function>().clone();
        closure.forget();
        Reflect::set(&global, &"pmem_1".into(), &func).unwrap();

        let cart_clone = cart.clone();
        let closure = Closure::wrap(Box::new(move |a, v| cart_clone.borrow_mut().set_pmem(a, v)) as Box<dyn FnMut(usize, i32) -> i32>);
        let func = closure.as_ref().unchecked_ref::<Function>().clone();
        closure.forget();
        Reflect::set(&global, &"pmem_2".into(), &func).unwrap();

        let cart_clone = cart.clone();
        let closure = Closure::wrap(Box::new(move |a, v| cart_clone.borrow_mut().poke(a, v)) as Box<dyn FnMut(usize, u8)>);
        let func = closure.as_ref().unchecked_ref::<Function>().clone();
        closure.forget();
        Reflect::set(&global, &"poke_2".into(), &func).unwrap();

        let cart_clone = cart.clone();
        let closure = Closure::wrap(Box::new(move |a, v, b| cart_clone.borrow_mut().poke_with_bits(a, v, b)) as Box<dyn FnMut(usize, u8, usize)>);
        let func = closure.as_ref().unchecked_ref::<Function>().clone();
        closure.forget();
        Reflect::set(&global, &"poke_3".into(), &func).unwrap();

        let cart_clone = cart.clone();
        let closure = Closure::wrap(Box::new(move |a, v| cart_clone.borrow_mut().poke4(a, v)) as Box<dyn FnMut(usize, u8)>);
        let func = closure.as_ref().unchecked_ref::<Function>().clone();
        closure.forget();
        Reflect::set(&global, &"poke4".into(), &func).unwrap();

        let cart_clone = cart.clone();
        let closure = Closure::wrap(Box::new(move |a, v| cart_clone.borrow_mut().poke2(a, v)) as Box<dyn FnMut(usize, u8)>);
        let func = closure.as_ref().unchecked_ref::<Function>().clone();
        closure.forget();
        Reflect::set(&global, &"poke2".into(), &func).unwrap();

        let cart_clone = cart.clone();
        let closure = Closure::wrap(Box::new(move |a, v| cart_clone.borrow_mut().poke1(a, v)) as Box<dyn FnMut(usize, u8)>);
        let func = closure.as_ref().unchecked_ref::<Function>().clone();
        closure.forget();
        Reflect::set(&global, &"poke1".into(), &func).unwrap();

        let cart_clone = cart.clone();
        let closure = Closure::wrap(Box::new(move |t: String, x, y, c: i32, f, s, a| cart_clone.borrow_mut().print(t.as_str(), x, y, c.try_into().unwrap_or(255), f, s, a)) as Box<dyn FnMut(String, i32, i32, i32, bool, i32, bool) -> i32>);
        let func = closure.as_ref().unchecked_ref::<Function>().clone();
        closure.forget();
        Reflect::set(&global, &"print_7".into(), &func).unwrap();

        let cart_clone = cart.clone();
        let closure = Closure::wrap(Box::new(move |t: String, x, y, c: i32, f, s, a| cart_clone.borrow_mut().print_ch(t.as_str(), x, y, c.try_into().unwrap_or(255), f, s, a)) as Box<dyn FnMut(String, i32, i32, i32, bool, i32, bool) -> i32>);
        let func = closure.as_ref().unchecked_ref::<Function>().clone();
        closure.forget();
        Reflect::set(&global, &"print_ch_7".into(), &func).unwrap();

        let cart_clone = cart.clone();
        let closure = Closure::wrap(Box::new(move |x, y, w, h, c: i32| cart_clone.borrow_mut().rect(x, y, w, h, c.try_into().unwrap_or(255))) as Box<dyn FnMut(i32, i32, i32, i32, i32)>);
        let func = closure.as_ref().unchecked_ref::<Function>().clone();
        closure.forget();
        Reflect::set(&global, &"rect".into(), &func).unwrap();

        let cart_clone = cart.clone();
        let closure = Closure::wrap(Box::new(move |x, y, w, h, c: i32| cart_clone.borrow_mut().rectb(x, y, w, h, c.try_into().unwrap_or(255))) as Box<dyn FnMut(i32, i32, i32, i32, i32)>);
        let func = closure.as_ref().unchecked_ref::<Function>().clone();
        closure.forget();
        Reflect::set(&global, &"rectb".into(), &func).unwrap();

        // todo: reset, sfx

        let cart_clone = cart.clone();
        let closure = Closure::wrap(Box::new(move |vec: Vec<i32>| cart_clone.borrow_mut().spr(vec[0], vec[1], vec[2], vec[3].try_into().unwrap_or(255), vec[4], vec[5], vec[6], vec[7], vec[8])) as Box<dyn FnMut(Vec<i32>)>);
        let func = closure.as_ref().unchecked_ref::<Function>().clone();
        closure.forget();
        Reflect::set(&global, &"spr_vec_9".into(), &func).unwrap();

        // todo: sync

        let cart_clone = cart.clone();
        let closure = Closure::wrap(Box::new(move |vec: Vec<f32>, m, c: i32| cart_clone.borrow_mut().textri(vec[0], vec[1], vec[2], vec[3], vec[4], vec[5], vec[6], vec[7], vec[8], vec[9], vec[10], vec[11], m, c.try_into().unwrap_or(255))) as Box<dyn FnMut(Vec<f32>, bool, i32)>);
        let func = closure.as_ref().unchecked_ref::<Function>().clone();
        closure.forget();
        Reflect::set(&global, &"textri_3".into(), &func).unwrap();

        let sys_clone = system.clone();
        let closure = Closure::wrap(Box::new(move || sys_clone.borrow().time()) as Box<dyn FnMut() -> u64>);
        let func = closure.as_ref().unchecked_ref::<Function>().clone();
        closure.forget();
        Reflect::set(&global, &"time".into(), &func).unwrap();

        let sys_clone = system.clone();
        let closure = Closure::wrap(Box::new(move |msg: JsValue, c| {sys_clone.borrow_mut().trace(msg.as_string().unwrap().as_str(), c);}) as Box<dyn FnMut(JsValue, u8)>);
        let func = closure.as_ref().unchecked_ref::<Function>().clone();
        closure.forget();
        Reflect::set(&global, &"trace_2".into(), &func).unwrap();

        let cart_clone = cart.clone();
        let closure = Closure::wrap(Box::new(move |x1, y1, x2, y2, x3, y3, c: i32| cart_clone.borrow_mut().tri(x1, y1, x2, y2, x3, y3, c.try_into().unwrap_or(255))) as Box<dyn FnMut(f32, f32, f32, f32, f32, f32, i32)>);
        let func = closure.as_ref().unchecked_ref::<Function>().clone();
        closure.forget();
        Reflect::set(&global, &"tri".into(), &func).unwrap();

        let cart_clone = cart.clone();
        let closure = Closure::wrap(Box::new(move |x1, y1, x2, y2, x3, y3, c: i32| cart_clone.borrow_mut().trib(x1, y1, x2, y2, x3, y3, c.try_into().unwrap_or(255))) as Box<dyn FnMut(f32, f32, f32, f32, f32, f32, i32)>);
        let func = closure.as_ref().unchecked_ref::<Function>().clone();
        closure.forget();
        Reflect::set(&global, &"trib".into(), &func).unwrap();

        let sys_clone = system.clone();
        let closure = Closure::wrap(Box::new(move || sys_clone.borrow().tstamp()) as Box<dyn FnMut() -> u64>);
        let func = closure.as_ref().unchecked_ref::<Function>().clone();
        closure.forget();
        Reflect::set(&global, &"tstamp".into(), &func).unwrap();

        let cart_clone = cart.clone();
        let closure = Closure::wrap(Box::new(move |i| {
            if i == 0 || i == 1 {
                cart_clone.borrow_mut().vbank(i);
            }
        }) as Box<dyn FnMut(usize)>);
        let func = closure.as_ref().unchecked_ref::<Function>().clone();
        closure.forget();
        Reflect::set(&global, &"vbank".into(), &func).unwrap();
    }
    fn load(&mut self, script: &str) -> Result<(), String> {
        let global = js_sys::global();
    
        let eval = Reflect::get(&global, &JsValue::from_str("eval")).map_err(|e| format!("Error accessing eval: {:?}", e))?
            .dyn_into::<Function>().map_err(|e| format!("Error converting eval to Function: {:?}", e))?;
        
        eval.call1(&JsValue::NULL, &JsValue::from_str(script)).map_err(|e| format!("Error executing script: {:?}", e))?;
        
        Ok(())
    }
    fn init(&mut self) -> Result<(), String> {
        js_sys::eval("if (typeof init !== 'undefined') { init(); }").map_err(|e| format!("Error calling init: {:?}", e))?;
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
