use crate::{
    system::SystemContext,
    cartridge::{
        CartContext,
        ram::{Vram, Ram},
    },
};
use js_sys::Date;
use std::{
    cell::RefCell,
    rc::Rc,
    collections::HashMap,
};

pub struct WheelWrapper {
    cart: Rc<RefCell<CartContext>>,
    system: Rc<RefCell<SystemContext>>,
    pub programs: HashMap<String, Rc<RefCell<dyn InternalProgram>>>,
    program: Option<Rc<RefCell<dyn InternalProgram>>>,
    file_buffer: Vec<u8>,
}

impl WheelWrapper {
    const BORDER_W: usize = 8;
    const BORDER_H: usize = 4;
    pub fn new() -> Self {
        Self {
            cart: Rc::new(RefCell::new(CartContext::new())),
            system: Rc::new(RefCell::new(SystemContext::new())),
            programs: HashMap::new(),
            program: None,
            file_buffer: Vec::new(),
        }
    }
    fn self_update(&mut self) {
        if let Some(program) = &self.program {
            program.borrow_mut().update();
            let exit_flag = self.system.borrow().exit_flag;
            if exit_flag {
                self.program = None;
                self.system.borrow_mut().exit_flag = false;
            }
            return;
        }
        self.cart.borrow_mut().cls(0);
        for i in self.system.borrow().top_line..self.system.borrow().lines.len() {
            self.cart.borrow_mut().print_ch(&self.system.borrow().lines[i], 0, (i - self.system.borrow().top_line) as i32 * 9, 13, true, 1, false);
        }
        let input_lines = SystemContext::split_line(&(">".to_string() + &self.system.borrow().input_buffer));
        for i in 0..input_lines.len() {
            self.cart.borrow_mut().print_ch(&input_lines[i], 0, (self.system.borrow().lines.len() - self.system.borrow().top_line) as i32 * 9 + i as i32 * 9, 13, true, 1, false);
        }
        //context.print_ch(, 0, (self_context.lines.len() - self_context.top_line) as i32 * 9, 13, true, 1, false);
        if self.cart.borrow().keyp(Some(62)) {
            let new_capslock = !self.system.borrow().capslock;
            self.system.borrow_mut().capslock = new_capslock;
        }
        if let Some(c) = self.get_char() {
            self.system.borrow_mut().scroll_to_bottom();
            self.system.borrow_mut().input_buffer.push(c);
        }
        if self.cart.borrow().keyp_with_hold_period(50, 60, 5) {
            self.system.borrow_mut().lines.extend(input_lines);
            let in_str = self.system.borrow().input_buffer.clone();
            match in_str.as_str() {
                "" => {},
                "clear" => self.system.borrow_mut().lines.clear(),
                "cls" => self.system.borrow_mut().lines.clear(),
                "run" => {
                    self.system.borrow_mut().program_timer = Date::now() as u64;
                    self.programs["demo"].borrow_mut().init(self.cart.clone(), self.system.clone());
                    self.program = Some(self.programs["demo"].clone());
                },
                "save" => self.file_buffer = self.programs["demo"].borrow().to_file().unwrap_or_default(),
                _ => {
                    self.system.borrow_mut().lines.push("未知命令".to_string());
                },
            }
            self.system.borrow_mut().input_buffer.clear();
            self.system.borrow_mut().scroll_to_bottom();
        }
        if self.cart.borrow().keyp_with_hold_period(51, 60, 5) {
            self.system.borrow_mut().input_buffer.pop();
        }
        let (_, _, _, _, _, _, sy) = self.cart.borrow().mouse();
        if sy > 0 {
            self.system.borrow_mut().scroll(1);
        } else if sy < 0 {
            self.system.borrow_mut().scroll(-1);
        }
    }
    fn get_color(&self, color: u8) -> u32 {
        let context = self.cart.borrow();
        let true_index = context.peek4(Vram::PALETTE_MAP_OFFSET * 2 + color as usize) as usize;
        let r = context.peek(Vram::PALETTE_OFFSET + true_index * 3) as u32;
        let g = context.peek(Vram::PALETTE_OFFSET + true_index * 3 + 1) as u32;
        let b = context.peek(Vram::PALETTE_OFFSET + true_index * 3 + 2) as u32;
        (r << 16) | (g << 8) | b
    }
    fn get_char(&self) -> Option<char> {
        const KEYBOARD_OFFSET: usize = crate::cartridge::ram::Ram::KEYBOARD_OFFSET;
        const NUM_SHIFTS: [char; 10] = [')', '!', '@', '#', '$', '%', '^', '&', '*', '('];
        let keys: Vec<u8> = (0..4).map(|i| self.cart.borrow().peek(KEYBOARD_OFFSET + i)).collect();
        let shift = keys.contains(&64) || self.system.borrow().capslock;
        for i in keys {
            if self.cart.borrow().keyp_with_hold_period(i, 60, 5) {
                let c = match i {
                    1..=26 => (i - 1 + if shift { b'A' } else { b'a' }) as char,
                    27..=36 => if shift { NUM_SHIFTS[(i - 27) as usize] } else { (i - 27 + b'0') as char },
                    37 => if shift { '_' } else { '-' },
                    38 => if shift { '+' } else { '=' },
                    39 => if shift { '{' } else { '[' },
                    40 => if shift { '}' } else { ']' },
                    41 => if shift { '|' } else { '\\' },
                    42 => if shift { ':' } else { ';' },
                    43 => if shift { '"' } else { '\'' },
                    44 => if shift { '~' } else { '`' },
                    45 => if shift { '<' } else { ',' },
                    46 => if shift { '>' } else { '.' },
                    47 => if shift { '?' } else { '/' },
                    48 => ' ',
                    49 => '\t',
                    79..=88 => (i - 79 + b'0') as char,
                    89 => '+',
                    90 => '-',
                    91 => '*',
                    92 => '/',
                    94 => '.',
                    _ => continue,
                };
                return Some(c);
            }
        }
        None
    }
}

pub trait InternalProgram {
    fn init(&mut self, cart: Rc<RefCell<CartContext>>, system: Rc<RefCell<SystemContext>>);
    fn update(&mut self);
    fn scanline(&mut self, _i: usize) {}
    fn overlay(&mut self) {}
    fn to_file(&self) -> Option<Vec<u8>> {
        None
    }
}

fn draw_fat_pixel(wheel: &mut dyn crate::WheelInterface, x: i32, y: i32, color: u32) {
    wheel.draw_pixel(x * 2, y * 2, color);
    wheel.draw_pixel(x * 2 + 1, y * 2, color);
    wheel.draw_pixel(x * 2, y * 2 + 1, color);
    wheel.draw_pixel(x * 2 + 1, y * 2 + 1, color);
}
fn draw_sub_pixel(wheel: &mut dyn crate::WheelInterface, x: i32, y: i32, data: [u32; 4]) {
    if data[0] <= 0xFFFFFF {
        wheel.draw_pixel(x * 2, y * 2, data[0]);
    }
    if data[1] <= 0xFFFFFF {
        wheel.draw_pixel(x * 2 + 1, y * 2, data[1]);
    }
    if data[2] <= 0xFFFFFF {
        wheel.draw_pixel(x * 2, y * 2 + 1, data[2]);
    }
    if data[3] <= 0xFFFFFF {
        wheel.draw_pixel(x * 2 + 1, y * 2 + 1, data[3]);
    }
}

impl crate::WheelProgram for WheelWrapper {
    fn init(&mut self, _wheel: &mut dyn crate::WheelInterface) {}
    fn update(&mut self, wheel: &mut dyn crate::WheelInterface) {
        // get input
        let mut btns = wheel.get_buttons();
        let keys = wheel.get_keys();

        let mut gamepad_map = 0;
        for i in 0..8 {
            if keys.contains(&self.cart.borrow().peek(Ram::GAMEPAD_MAPPING_OFFSET + i)) {
                gamepad_map |= 1 << i;
            }
        }
        btns[0] |= gamepad_map;
        for i in 0..4 {
            self.cart.borrow_mut().poke(Ram::GAMEPADS_OFFSET + i, btns[i]);
            self.cart.borrow_mut().poke(Ram::KEYBOARD_OFFSET + i, keys[i]);
        }

        // use direct input to update, instead of ram data
        // correctness to be confirmed
        let btn_bin = u32::from_le_bytes(btns);
        for i in 0..32 {
            if (btn_bin & (1 << i)) == 0 {
                self.cart.borrow_mut().btn_timer[i] = -1;
            } else {
                self.cart.borrow_mut().btn_timer[i] += 1;
            }
        }
        let mut key_del_list = Vec::new();
        for (key, time) in self.cart.borrow_mut().key_timer.iter_mut() {
            if keys.contains(key) {
                *time += 1;
            } else {
                key_del_list.push(*key);
            }
        }
        for key in key_del_list {
            self.cart.borrow_mut().key_timer.remove(&key);
        }
        for key in keys {
            if !self.cart.borrow_mut().key_timer.contains_key(&key) {
                self.cart.borrow_mut().key_timer.insert(key, 0);
            }
        }
        let mouse = wheel.get_mouse();
        let mut mouse_x = 
            if mouse.x > (Self::BORDER_W * 2) as i32 && mouse.x <= 2 * (Vram::SCREEN_WIDTH + 2 * Self::BORDER_W) as i32 {
                mouse.x / 2 - Self::BORDER_W as i32
            } else {
                -1
            };
        let mut mouse_y = 
            if mouse.y > (Self::BORDER_H * 2) as i32 && mouse.y <= 2 * (Vram::SCREEN_HEIGHT + 2 * Self::BORDER_H) as i32 {
                mouse.y / 2 - Self::BORDER_H as i32
            } else {
                -1
            };
        if mouse_x == -1 || mouse_y == -1 {
            mouse_x = -1;
            mouse_y = -1;
        }
        self.cart.borrow_mut().poke(Ram::MOUSE_OFFSET, mouse_x as u8);
        self.cart.borrow_mut().poke(Ram::MOUSE_OFFSET + 1, mouse_y as u8);
        const SCROLL_FACTOR: i32 = 50;
        let mut mouse_lw: u16 = mouse.left as u16 | ((mouse.middle as u16) << 1) | ((mouse.right as u16) << 2);
        let scroll_x = mouse.scroll_x / SCROLL_FACTOR;
        let scroll_y = mouse.scroll_y / SCROLL_FACTOR;
        mouse_lw |= ((scroll_x & 0b111111) as u16) << 3;
        mouse_lw |= ((scroll_y & 0b111111) as u16) << 9;
        self.cart.borrow_mut().poke(Ram::MOUSE_OFFSET + 2, mouse_lw as u8);
        self.cart.borrow_mut().poke(Ram::MOUSE_OFFSET + 3, (mouse_lw >> 8) as u8);

        // draw screen
        let x = self.cart.borrow_mut().active_bank.clone();
        self.cart.borrow_mut().ram[x].set_active_vbank(0);
        self.self_update();
        for i in 0..Self::BORDER_H {
            if let Some(prog) = &self.program {
                prog.borrow_mut().scanline(i);
            }
            let color = self.get_color(self.cart.borrow().peek(Vram::BORDER_COLOR_OFFSET));
            for x in 0..(Self::BORDER_W * 2 + Vram::SCREEN_WIDTH) as i32 {
                draw_fat_pixel(wheel, x, i as i32, color);
            }
        }
        for i in 0..Vram::SCREEN_HEIGHT {
            if let Some(prog) = &self.program {
                prog.borrow_mut().scanline(i + Self::BORDER_H);
            }
            let palette: Vec<u32> = (0..16).into_iter().map(|c| self.get_color(c)).collect();
            let x_offset: i32 = (self.cart.borrow().peek(Vram::SCREEN_OFFSET_OFFSET) as i8).into();
            let y_offset: i32 = (self.cart.borrow().peek(Vram::SCREEN_OFFSET_OFFSET + 1) as i8).into();
            let y = (i as i32 + y_offset) % Vram::SCREEN_HEIGHT as i32 + Self::BORDER_H as i32;
            for xx in 0..Vram::SCREEN_WIDTH {
                let color = palette[self.cart.borrow().peek4(i * Vram::SCREEN_WIDTH + xx) as usize];
                let x = (xx as i32 + x_offset) % Vram::SCREEN_WIDTH as i32 + Self::BORDER_W as i32;
                draw_fat_pixel(wheel, x, y, color);
                let subpix = self.cart.borrow_mut().get_subpix_map_mut().get(xx, i);
                if let Some(arr) = subpix {
                    let mut colors = [0; 4];
                    for j in 0..4 {
                        colors[j] = if arr[j] < 16 { palette[arr[j] as usize] } else { 0xFFFFFFFF };
                    }
                    draw_sub_pixel(wheel, x, y, colors);
                }
            }
            let color = self.get_color(self.cart.borrow().peek(Vram::BORDER_COLOR_OFFSET));
            for x in 0..Self::BORDER_W as i32 {
                draw_fat_pixel(wheel, x, y, color);
                let x = x + (Self::BORDER_W + Vram::SCREEN_WIDTH) as i32;
                draw_fat_pixel(wheel, x, y, color);
            }
        }
        for i in 0..Self::BORDER_H {
            let ii = i + Self::BORDER_H + Vram::SCREEN_HEIGHT;
            if let Some(prog) = &self.program {
                prog.borrow_mut().scanline(ii);
            }
            let color = self.get_color(self.cart.borrow().peek(Vram::BORDER_COLOR_OFFSET));
            for x in 0..(Self::BORDER_W * 2 + Vram::SCREEN_WIDTH) as i32 {
                draw_fat_pixel(wheel, x, ii as i32, color);
            }
        }

        let x = self.cart.borrow_mut().active_bank.clone();
        self.cart.borrow_mut().ram[x].set_active_vbank(1);
        if let Some(prog) = &self.program {
            prog.borrow_mut().overlay();
        }
        let palette: Vec<u32> = (0..16).into_iter().map(|c| self.get_color(c)).collect();
        let trans_color = self.cart.borrow().peek(Vram::BORDER_COLOR_OFFSET) & 0xf;
        let x_offset: i32 = (self.cart.borrow().peek(Vram::SCREEN_OFFSET_OFFSET) as i8).into();
        let y_offset: i32 = (self.cart.borrow().peek(Vram::SCREEN_OFFSET_OFFSET + 1) as i8).into();
        for i in 0..Vram::SCREEN_HEIGHT{
            let y = (i as i32 + y_offset) % Vram::SCREEN_HEIGHT as i32 + Self::BORDER_H as i32;
            for xx in 0..Vram::SCREEN_WIDTH {
                let color_id = self.cart.borrow().peek4(i * Vram::SCREEN_WIDTH + xx);
                if color_id != trans_color {
                    let color = palette[color_id as usize];
                    let x = (xx as i32 + x_offset) % Vram::SCREEN_WIDTH as i32 + Self::BORDER_W as i32;
                    draw_fat_pixel(wheel, x, y, color);
                    let subpix = self.cart.borrow_mut().get_subpix_map_mut().get(xx, i);
                    if let Some(arr) = subpix {
                        let mut colors = [0; 4];
                        for j in 0..4 {
                            colors[j] = if arr[j] < 16 { palette[arr[j] as usize] } else { 0xFFFFFFFF };
                        }
                        draw_sub_pixel(wheel, x, y, colors);
                    }
                }
            }
        }

        // todo: play sound

        if !self.file_buffer.is_empty() {
            wheel.save_file("demo.wheel", self.file_buffer.clone());
            self.file_buffer.clear();
        }
    }
}
