pub mod ram;
use crate::cartridge::ram::{Vram, Ram};
use std::collections::{HashSet, HashMap};

pub struct CartContext {
    ram: Vec<Ram>,
    active_bank: usize,
    clip_rect: (i32, i32, i32, i32),
    trans_map: HashSet<(i32, i32)>,
    key_timer: HashMap<u8, u32>,
    btn_timer: [i32; 32],
}

impl CartContext {
    const BANK_N: usize = 8;
    pub fn new() -> Self {
        Self {
            ram: vec![Ram::new(); Self::BANK_N],
            active_bank: 0,
            clip_rect: (0, 0, Vram::SCREEN_WIDTH as i32, Vram::SCREEN_HEIGHT as i32),
            trans_map: HashSet::new(),
            key_timer: HashMap::new(),
            btn_timer: [0; 32],
        }
    }

    // memory manipulations

    pub fn memcpy(&mut self, from: usize, to: usize, length: usize) {
        let buffer: Vec<u8> = (from..from + length).into_iter().map(|x| self.ram[self.active_bank][x]).collect();
        for i in 0..length {
            self.ram[self.active_bank][to + i] = buffer[i];
        }
    }
    pub fn memset(&mut self, addr: usize, value: u8, length: usize) {
        for i in 0..length {
            self.ram[self.active_bank][addr + i] = value;
        }
    }
    pub fn peek_with_bits(&self, addr: usize, bits: usize) -> u8 {
        (self.ram[self.active_bank][addr * bits / 8] >> (bits * (addr % (8 / bits)))) & ((1u16 << bits) - 1) as u8
    }
    pub fn peek(&self, addr: usize) -> u8 {
        self.ram[self.active_bank][addr]
    }
    pub fn peek4(&self, addr: usize) -> u8 {
        (self.ram[self.active_bank][addr / 2] >> (4 * (addr % 2))) & 0xf
    }
    pub fn peek2(&self, addr: usize) -> u8 {
        (self.ram[self.active_bank][addr / 4] >> (2 * (addr % 4))) & 0b11
    }
    pub fn peek1(&self, addr: usize) -> u8 {
        (self.ram[self.active_bank][addr / 8] >> (1 * (addr % 8))) & 0b1
    }
    pub fn poke_with_bits(&mut self, addr: usize, val: u8, bits: usize) {
        let byte_borrow = &mut self.ram[self.active_bank][addr * bits / 8];
        let bit_offset = ((addr % (8 / bits)) * bits) as u8;
        let mask: u8 = ((((1u16 << bits) - 1) as u8) << bit_offset) ^ 0xff;
        let val_mask = val << bit_offset;
        *byte_borrow &= mask;
        *byte_borrow |= val_mask;
    }
    pub fn poke(&mut self, addr: usize, val: u8) {
        self.ram[self.active_bank][addr] = val;
    }
    pub fn poke4(&mut self, addr: usize, val: u8) {
        self.poke_with_bits(addr, val, 4);
    }
    pub fn poke2(&mut self, addr: usize, val: u8) {
        self.poke_with_bits(addr, val, 2);
    }
    pub fn poke1(&mut self, addr: usize, val: u8) {
        self.poke_with_bits(addr, val, 1);
    }
    pub fn get_pmem(&self, index: usize) -> i32 {
        if index < Ram::PERSISTENT_MEMORY_SIZE {
            let addr = Ram::PERSISTENT_MEMORY_OFFSET + 4 * index;
            let mut res: i32 = 0;
            for i in 0..4 {
                res |= (self.ram[self.active_bank][addr + i] as i32) << (i * 8);
            }
            res
        } else {
            0
        }
    }
    pub fn set_pmem(&mut self, index: usize, val: i32) -> i32 {
        if index < Ram::PERSISTENT_MEMORY_SIZE {
            let addr = Ram::PERSISTENT_MEMORY_OFFSET + 4 * index;
            let mut res: i32 = 0;
            for i in 0..4 {
                res |= (self.ram[self.active_bank][addr + i] as i32) << (i * 8);
                self.ram[self.active_bank][addr + i] = ((val >> (i * 8)) & 0xff) as u8;
            }
            res
        } else {
            0
        }
    }
    pub fn vbank(&mut self, id: usize) {
        if id < Vram::VBANK_N {
            self.ram[self.active_bank].set_active_vbank(id);
        }
    }

    // graphics

    fn in_clip(&self, x: i32, y: i32) -> bool {
        x >= self.clip_rect.0 && x < self.clip_rect.0 + self.clip_rect.2
            && y >= self.clip_rect.1 && y < self.clip_rect.1 + self.clip_rect.3
    }
    pub fn clip(&mut self, x: i32, y: i32, w: i32, h: i32) {
        self.clip_rect.0 = x.max(0);
        self.clip_rect.1 = y.max(0);
        self.clip_rect.2 = w.min(Vram::SCREEN_WIDTH as i32 - self.clip_rect.0);
        self.clip_rect.3 = h.min(Vram::SCREEN_HEIGHT as i32 - self.clip_rect.1);
    }

    fn draw_while(&mut self, from: i32, to: i32, color: u8, coord: &dyn Fn(i32) -> (i32, i32)) {
        if color < 16 {
            for i in from..to {
                let (x, y) = coord(i);
                if !self.in_clip(x, y) {
                    break;
                }
                self.poke4(y as usize * Vram::SCREEN_WIDTH + x as usize, color);
            }
        }
    }

    pub fn set_pix(&mut self, x: i32, y: i32, color: u8) {
        if self.in_clip(x, y) && color < 16 {
            self.poke4(y as usize * Vram::SCREEN_WIDTH + x as usize, color);
        }
    }
    pub fn get_pix(&mut self, x: i32, y: i32) -> u8 {
        if self.in_clip(x, y) {
            self.peek4(y as usize * Vram::SCREEN_WIDTH + x as usize)
        } else {
            255
        }
    }
    pub fn rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: u8) {
        let x = x.max(self.clip_rect.0);
        let y = y.max(self.clip_rect.1);
        let w = w.min(self.clip_rect.0 + self.clip_rect.2 - x);
        let h = h.min(self.clip_rect.1 + self.clip_rect.3 - y);
        for yy in y..y + h {
            for xx in x..x + w {
                self.poke4(yy as usize * Vram::SCREEN_WIDTH + xx as usize, color);
            }
        }
    }
    pub fn rectb(&mut self, x: i32, y: i32, w: i32, h: i32, color: u8) {
        let x_start = x.max(self.clip_rect.0);
        let y_start = y.max(self.clip_rect.1);
        self.draw_while(x_start, x + w, color, &(|x| (x, y)));
        self.draw_while(x_start, x + w, color, &(|x| (x, y + h - 1)));
        self.draw_while(y_start, y + h, color, &(|y| (x, y)));
        self.draw_while(y_start, y + h, color, &(|y| (x + w - 1, y)));
    }
    fn hline(&mut self, x: i32, y: i32, w: i32, color: u8) {
        let x_start = x.max(self.clip_rect.0);
        self.draw_while(x_start, x + w, color, &(|x| (x, y)));
    }
    fn vline(&mut self, x: i32, y: i32, h: i32, color: u8) {
        let y_start = y.max(self.clip_rect.1);
        self.draw_while(y_start, y + h, color, &(|y| (x, y)));
    }
    pub fn line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, color: u8) {
        let (x1, x2, y1, y2) = if x1 > x2 { (x2, x1, y2, y1) } else { (x1, x2, y1, y2) };
        let dx = x2 - x1;
        let dy = y2 - y1;
        let mut xi = x1.floor();
        let mut yi;
        if dy < 0.0 {
            yi = y1.ceil();
            if -dy > dx {
                while yi >= y2 {
                    self.set_pix(xi as i32, yi as i32, color);
                    yi -= 1.0;
                    if -(dx * yi - dy * xi - dx * y1 + dy * x1) > dx * yi - dy * (xi + 1.0) - dx * y1 + dy * x1 {
                        xi += 1.0;
                    }
                }
            } else {
                while xi <= x2 {
                    self.set_pix(xi as i32, yi as i32, color);
                    xi += 1.0;
                    if dx * yi - dy * xi - dx * y1 + dy * x1 > -(dx * (yi - 1.0) - dy * xi - dx * y1 + dy * x1) {
                        yi -= 1.0;
                    }
                }
            }
        } else {
            yi = y1.floor();
            if dy > dx {
                while yi <= y2 {
                    self.set_pix(xi as i32, yi as i32, color);
                    yi += 1.0;
                    if dx * yi - dy * xi - dx * y1 + dy * x1 > -(dx * yi - dy * (xi + 1.0) - dx * y1 + dy * x1) {
                        xi += 1.0;
                    }
                }
            } else {
                while xi <= x2 {
                    self.set_pix(xi as i32, yi as i32, color);
                    xi += 1.0;
                    if -(dx * yi - dy * xi - dx * y1 + dy * x1) > dx * (yi + 1.0) - dy * xi - dx * y1 + dy * x1 {
                        yi += 1.0;
                    }
                }
            }
        }
    }
    pub fn circ(&mut self, x: i32, y: i32, r: i32, color: u8) {
        if r > 0 {
            self.hline(x - r, y, r * 2 + 1, color);
            self.vline(x, y - r, r * 2 + 1, color);
            let mut rx: i32 = r;
            let mut ry: i32 = 0;
            while rx > ry + 1 {
                ry += 1;
                if rx * rx + ry * ry - r * r > -((rx - 1) * (rx - 1) + ry * ry - r * r) {
                    rx -= 1;
                }
                self.hline(x - rx, y + ry, rx * 2 + 1, color);
                self.hline(x - rx, y - ry, rx * 2 + 1, color);
                self.hline(x - ry, y + rx, ry * 2 + 1, color);
                self.hline(x - ry, y - rx, ry * 2 + 1, color);
            }
        }
    }
    pub fn circb(&mut self, x: i32, y: i32, r: i32, color: u8) {
        if r > 0 {
            self.set_pix(x + r, y, color);
            self.set_pix(x - r, y, color);
            self.set_pix(x, y + r, color);
            self.set_pix(x, y - r, color);
            let mut rx: i32 = r;
            let mut ry: i32 = 0;
            while rx > ry + 1 {
                ry += 1;
                if rx * rx + ry * ry - r * r > -((rx - 1) * (rx - 1) + ry * ry - r * r) {
                    rx -= 1;
                }
                self.set_pix(x + rx, y + ry, color);
                self.set_pix(x + rx, y - ry, color);
                self.set_pix(x - rx, y + ry, color);
                self.set_pix(x - rx, y - ry, color);
                self.set_pix(x + ry, y + rx, color);
                self.set_pix(x + ry, y - rx, color);
                self.set_pix(x - ry, y + rx, color);
                self.set_pix(x - ry, y - rx, color);
            }
        }
    }
    fn elli_d(rx: i32, ry: i32, a: i32, b: i32) -> i32 {
        (rx * rx * b * b + ry * ry * a * a - a * a * b * b).abs()
    }
    pub fn elli(&mut self, x: i32, y: i32, a: i32, b: i32, color: u8) {
        if a > 0 && b > 0 && color < 16 {
            let mut rx = a;
            let mut ry = 0;
            while ry < b {
                self.hline(x - rx, y + ry, rx * 2 + 1, color);
                self.hline(x - rx, y - ry, rx * 2 + 1, color);
                if Self::elli_d(rx - 1, ry + 1, a, b) < Self::elli_d(rx, ry + 1, a, b) {
                    rx -= 1;
                    if Self::elli_d(rx - 1, ry + 1, a, b) < Self::elli_d(rx - 1, ry, a, b) {
                        ry += 1;
                    }
                } else {
                    ry += 1;
                }
            }
            self.hline(x - rx, y + b, rx * 2 + 1, color);
            self.hline(x - rx, y - b, rx * 2 + 1, color);
        }
    }
    pub fn ellib(&mut self, x: i32, y: i32, a: i32, b: i32, color: u8) {
        if a > 0 && b > 0 && color < 16 {
            let mut rx = a;
            let mut ry = 0;
            while ry < b {
                self.set_pix(x + rx, y + ry, color);
                self.set_pix(x + rx, y - ry, color);
                self.set_pix(x - rx, y + ry, color);
                self.set_pix(x - rx, y - ry, color);
                if Self::elli_d(rx - 1, ry + 1, a, b) < Self::elli_d(rx, ry + 1, a, b) {
                    rx -= 1;
                    if Self::elli_d(rx - 1, ry + 1, a, b) < Self::elli_d(rx - 1, ry, a, b) {
                        ry += 1;
                    }
                } else {
                    ry += 1;
                }
            }
            self.hline(x - rx, y + b, rx * 2 + 1, color);
            self.hline(x - rx, y - b, rx * 2 + 1, color);
        }
    }
    pub fn cls(&mut self, color: u8) {
        for y in self.clip_rect.1 .. self.clip_rect.1 + self.clip_rect.3 {
            for x in self.clip_rect.0 .. self.clip_rect.0 + self.clip_rect.2 {
                self.poke4(y as usize * Vram::SCREEN_WIDTH + x as usize, color);
            }
        }
    }

    fn putchar(&mut self, chr: u8, x: i32, y: i32, color: u8, is_fixed: bool, scale: i32, alt_font: bool) -> i32 {
        let chr: usize = (chr % 128).into(); // ascii characters only
        let font_offset = if alt_font { Ram::ALT_FONT_OFFSET } else { Ram::SYSTEM_FONT_OFFSET };
        if is_fixed {
            for i in 0..8 {
                let line_data = self.peek(font_offset + chr * 8 + i);
                for j in 0..8 {
                    if ((line_data >> j) & 1) != 0 {
                        self.rect(x + j as i32 * scale, y + i as i32 * scale, scale, scale, color);
                    }
                }
            }
            self.peek(font_offset + Ram::FONT_PARAM_OFFSET_RELATIVE).into()
        } else {
            let mut chr_bin = [0; 8];
            let mut max_bit = 0;
            let mut min_bit = 7;
            let mut zero_flag = true;
            for i in 0..8 {
                chr_bin[i] = self.peek(font_offset + chr * 8 + i);
                if chr_bin[i] != 0 {
                    zero_flag = false;
                    max_bit = max_bit.max(chr_bin[i].ilog2() as i32);
                    min_bit = min_bit.min(chr_bin[i].trailing_zeros() as i32);
                }
            }
            for i in 0..8 {
                for j in 0..(8 - min_bit) {
                    if ((chr_bin[i] >> (j + min_bit)) & 1) != 0 {
                        self.rect(x + j as i32 * scale, y + i as i32 * scale, scale, scale, color);
                    }
                }
            }
            scale * (if zero_flag { self.peek(font_offset + Ram::FONT_PARAM_OFFSET_RELATIVE) as i32 - 2 } else { max_bit - min_bit + 2 })
        }
    }

    pub fn print(&mut self, text: &str, x: i32, y: i32, color: u8, is_fixed: bool, scale: i32, alt_font: bool) -> i32 {
        let mut text_width = 0;
        let mut x_offset = 0;
        let mut y = y;
        let chr_height: i32 = scale * self.peek((if alt_font { Ram::ALT_FONT_OFFSET } else { Ram::SYSTEM_FONT_OFFSET }) + Ram::FONT_PARAM_OFFSET_RELATIVE + 1) as i32;
        for &chr in text.as_bytes() {
            if chr == '\n' as u8 {
                text_width = text_width.max(x_offset);
                x_offset = 0;
                y += chr_height;
            } else {
                x_offset += self.putchar(chr, x + x_offset, y, color, is_fixed, scale, alt_font);
            }
        }
        text_width.max(x_offset)
    }

    // inputs

    pub fn btn(&self, id: u8) -> bool {
        id < 32 && self.peek1(Ram::GAMEPADS_OFFSET * 8 + id as usize) == 1
    }
    pub fn btnp(&self, id: u8) -> bool {
        id < 32 && self.btn_timer[id as usize] == 0
    }
    pub fn btnp_with_hold_period(&self, id: u8, hold: i32, period: i32) -> bool {
        let hold = hold.max(0);
        let period = period.max(1);
        id < 32 && {
            let time = self.btn_timer[id as usize];
            time == 0 || (time > hold && (time - hold) % period == 0)
        }
    }
    pub fn key(&self, key_code: Option<u8>) -> bool {
        let key_buffer: Vec<u8> = (0..4).into_iter().map(|i| self.peek(Ram::KEYBOARD_OFFSET + i)).collect();
        match key_code {
            Some(code) => key_buffer.contains(&code),
            None => (key_buffer[0] | key_buffer[1] | key_buffer[2] | key_buffer[3]) != 0,
        }
    }
    pub fn keyp(&self, key_code: Option<u8>) -> bool {
        match key_code {
            Some(code) => self.key_timer.get(&code) == Some(&0),
            None => {
                let mut flag = false;
                for (_, time) in self.key_timer.iter() {
                    flag = flag || (*time == 0);
                }
                flag
            }
        }
    }
    pub fn keyp_with_hold_period(&self, key_code: u8, hold: i32, period: i32) -> bool {
        let hold = hold.max(0) as u32;
        let period = period.max(1) as u32;
        match self.key_timer.get(&key_code) {
            None => false,
            Some(&time) => time == 0 || (time > hold && (time - hold) % period == 0)
        }
    }
    pub fn mouse(&self) -> (u8, u8, bool, bool, bool, i8, i8) {
        let x = self.peek(Ram::MOUSE_OFFSET) as u8;
        let y = self.peek(Ram::MOUSE_OFFSET + 1) as u8;
        let res: u16 = self.peek(Ram::MOUSE_OFFSET + 2) as u16 | ((self.peek(Ram::MOUSE_OFFSET + 3) as u16) << 8);
        (x, y, (res & 1) != 0, (res & 2) != 0, (res & 4) != 0, ((res >> 1) as i8) >> 2, ((res >> 7) as i8) >> 2)
    }
}

pub trait CartProgram {
    fn init(&mut self, context: &mut CartContext) {}
    fn update(&mut self, context: &mut CartContext) {}
    fn scanline(&mut self, context: &mut CartContext, line_i: usize) {}
    fn overlay(&mut self, context: &mut CartContext) {}
}

pub struct Cartridge {
    context: CartContext,
    program: Box<dyn CartProgram>,
}

impl Cartridge {
    const BORDER_W: usize = 8;
    const BORDER_H: usize = 4;
    pub fn new(program: Box<dyn CartProgram>) -> Self {
        Self {
            context: CartContext::new(),
            program
        }
    }
    fn get_color(&self, color: u8) -> u32 {
        let true_index = self.context.peek4(Vram::PALETTE_MAP_OFFSET * 2 + color as usize) as usize;
        let r = self.context.peek(Vram::PALETTE_OFFSET + true_index * 3) as u32;
        let g = self.context.peek(Vram::PALETTE_OFFSET + true_index * 3 + 1) as u32;
        let b = self.context.peek(Vram::PALETTE_OFFSET + true_index * 3 + 2) as u32;
        (r << 16) | (g << 8) | b
    }
}

fn draw_fat_pixel(wheel: &mut dyn crate::WheelInterface, x: i32, y: i32, color: u32) {
    wheel.draw_pixel(x * 2, y * 2, color);
    wheel.draw_pixel(x * 2 + 1, y * 2, color);
    wheel.draw_pixel(x * 2, y * 2 + 1, color);
    wheel.draw_pixel(x * 2 + 1, y * 2 + 1, color);
}

impl crate::WheelProgram for Cartridge {
    fn init(&mut self, _wheel: &mut dyn crate::WheelInterface) {
        self.program.init(&mut self.context);
    }
    fn update(&mut self, wheel: &mut dyn crate::WheelInterface) {
        // get input
        let mut btns = wheel.get_buttons();
        let keys = wheel.get_keys();

        let mut gamepad_map = 0;
        for i in 0..8 {
            if keys.contains(&self.context.peek(Ram::GAMEPAD_MAPPING_OFFSET + i)) {
                gamepad_map |= 1 << i;
            }
        }
        btns[0] |= gamepad_map;
        for i in 0..4 {
            self.context.poke(Ram::GAMEPADS_OFFSET, btns[i]);
            self.context.poke(Ram::KEYBOARD_OFFSET, keys[i]);
        }

        // use direct input to update, instead of ram data
        // correctness to be confirmed
        let btn_bin = u32::from_le_bytes(btns);
        for i in 0..32 {
            if (btn_bin & (1 << i)) == 0 {
                self.context.btn_timer[i] = -1;
            } else {
                self.context.btn_timer[i] += 1;
            }
        }
        let mut key_del_list = Vec::new();
        for (key, time) in self.context.key_timer.iter_mut() {
            if keys.contains(key) {
                *time += 1;
            } else {
                key_del_list.push(*key);
            }
        }
        for key in key_del_list {
            self.context.key_timer.remove(&key);
        }
        for key in keys {
            if !self.context.key_timer.contains_key(&key) {
                self.context.key_timer.insert(key, 0);
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
        self.context.poke(Ram::MOUSE_OFFSET, mouse_x as u8);
        self.context.poke(Ram::MOUSE_OFFSET + 1, mouse_y as u8);
        const SCROLL_FACTOR: i32 = 50;
        let mut mouse_lw: u16 = mouse.left as u16 | ((mouse.middle as u16) << 1) | ((mouse.right as u16) << 2);
        let scroll_x = mouse.scroll_x / SCROLL_FACTOR;
        let scroll_y = mouse.scroll_y / SCROLL_FACTOR;
        mouse_lw |= ((scroll_x & 0b111111) as u16) << 3;
        mouse_lw |= ((scroll_y & 0b111111) as u16) << 9;
        self.context.poke(Ram::MOUSE_OFFSET + 2, mouse_lw as u8);
        self.context.poke(Ram::MOUSE_OFFSET + 3, (mouse_lw >> 8) as u8);

        self.program.update(&mut self.context);

        // draw screen
        self.context.ram[self.context.active_bank].set_active_vbank(1);
        self.program.overlay(&mut self.context);
        self.context.trans_map.clear();
        let palette: Vec<u32> = (0..16).into_iter().map(|c| self.get_color(c)).collect();
        let trans_color = self.context.peek(Vram::BORDER_COLOR_OFFSET) & 0xf;
        let x_offset: i32 = (self.context.peek(Vram::SCREEN_OFFSET_OFFSET) as i8).into();
        let y_offset: i32 = (self.context.peek(Vram::SCREEN_OFFSET_OFFSET + 1) as i8).into();
        for i in 0..Vram::SCREEN_HEIGHT{
            let y = (i as i32 + y_offset) % Vram::SCREEN_HEIGHT as i32 + Self::BORDER_H as i32;
            for xx in 0..Vram::SCREEN_WIDTH {
                let color_id = self.context.peek4(i * Vram::SCREEN_WIDTH + xx);
                if color_id != trans_color {
                    let color = palette[color_id as usize];
                    let x = (xx as i32 + x_offset) % Vram::SCREEN_WIDTH as i32 + Self::BORDER_W as i32;
                    draw_fat_pixel(wheel, x, y, color);
                    self.context.trans_map.insert((xx as i32, i as i32));
                }
            }
        }

        self.context.ram[self.context.active_bank].set_active_vbank(0);
        for i in 0..Self::BORDER_H {
            self.program.scanline(&mut self.context, i);
            let color = self.get_color(self.context.peek(Vram::BORDER_COLOR_OFFSET));
            for x in 0..(Self::BORDER_W * 2 + Vram::SCREEN_WIDTH) as i32 {
                draw_fat_pixel(wheel, x, i as i32, color);
            }
        }
        for i in 0..Vram::SCREEN_HEIGHT {
            self.program.scanline(&mut self.context, i + Self::BORDER_H);
            let palette: Vec<u32> = (0..16).into_iter().map(|c| self.get_color(c)).collect();
            let x_offset: i32 = (self.context.peek(Vram::SCREEN_OFFSET_OFFSET) as i8).into();
            let y_offset: i32 = (self.context.peek(Vram::SCREEN_OFFSET_OFFSET + 1) as i8).into();
            let y = (i as i32 + y_offset) % Vram::SCREEN_HEIGHT as i32 + Self::BORDER_H as i32;
            for xx in 0..Vram::SCREEN_WIDTH {
                let color = palette[self.context.peek4(i * Vram::SCREEN_WIDTH + xx) as usize];
                let x = (xx as i32 + x_offset) % Vram::SCREEN_WIDTH as i32 + Self::BORDER_W as i32;
                if !self.context.trans_map.contains(&(xx as i32, i as i32)) {
                    draw_fat_pixel(wheel, x, y, color);
                }
            }
            let color = self.get_color(self.context.peek(Vram::BORDER_COLOR_OFFSET));
            for x in 0..Self::BORDER_W as i32 {
                draw_fat_pixel(wheel, x, y, color);
                let x = x + (Self::BORDER_W + Vram::SCREEN_WIDTH) as i32;
                draw_fat_pixel(wheel, x, y, color);
            }
        }
        for i in 0..Self::BORDER_H {
            let ii = i + Self::BORDER_H + Vram::SCREEN_HEIGHT;
            self.program.scanline(&mut self.context, ii);
            let color = self.get_color(self.context.peek(Vram::BORDER_COLOR_OFFSET));
            for x in 0..(Self::BORDER_W * 2 + Vram::SCREEN_WIDTH) as i32 {
                draw_fat_pixel(wheel, x, ii as i32, color);
            }
        }

        // todo: play sound
    }
}

#[cfg(test)]
mod tests {
    use crate::cartridge::CartContext;
    use crate::cartridge::ram::{Vram, Ram};
    #[test]
    fn test_ram() {
        let mut context = CartContext::new();
        assert_eq!(Vram::BLIT_SEGMENT_OFFSET, 0x3ffc);
        assert_eq!(Ram::GAMEPAD_MAPPING_OFFSET, 0x14e04);
        //context.poke(0x12345, 0xab);
        context.poke_with_bits(0x12345 * 2, 0xb, 4);
        context.poke_with_bits(0x12345 * 2 + 1, 0xa, 4);
        assert_eq!(context.peek4(0x12345 * 2), context.peek_with_bits(0x12345*2, 4));
        assert_eq!(context.peek4(0x12345 * 2), 0xb);
        assert_eq!(context.peek4(0x12345 * 2 + 1), 0xa);
        context.set_pmem(12, 0x01234567);
        assert_eq!(context.get_pmem(12), 0x01234567);
        assert_eq!(context.peek(Ram::PERSISTENT_MEMORY_OFFSET + 12 * 4 + 1), 0x45);
        context.set_pmem(12, 0xfedc9999u32 as i32);
        assert_eq!(context.get_pmem(12), 0xfedc9999u32 as i32);

    }
}
