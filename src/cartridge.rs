pub mod ram;
use crate::cartridge::ram::{Vram, Ram};
use std::collections::{HashSet, HashMap};

pub struct CartContext {
    ram: Vec<Ram>,
    active_bank: usize,
    clip_rect: (i32, i32, i32, i32),
    trans_map: HashSet<(i32, i32)>,
    key_timer: HashMap<u8, u32>,
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
        }
    }

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
        for i in from..to {
            let (x, y) = coord(i);
            if !self.in_clip(x, y) {
                break;
            }
            self.poke4(y as usize * Vram::SCREEN_WIDTH + x as usize, color);
        }
    }

    pub fn set_pix(&mut self, x: i32, y: i32, color: u8) {
        if self.in_clip(x, y) {
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
    pub fn cls(&mut self, color: u8) {
        for y in self.clip_rect.1 .. self.clip_rect.1 + self.clip_rect.3 {
            for x in self.clip_rect.0 .. self.clip_rect.0 + self.clip_rect.2 {
                self.poke4(y as usize * Vram::SCREEN_WIDTH + x as usize, color);
            }
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
    fn init(&mut self, wheel: &mut dyn crate::WheelInterface) {
        self.program.init(&mut self.context);
    }
    fn update(&mut self, wheel: &mut dyn crate::WheelInterface) {
        // get input
        let btns = wheel.get_buttons();
        let keys = wheel.get_keys();
        for i in 0..4 {
            self.context.poke(Ram::GAMEPADS_OFFSET, btns[i]);
            self.context.poke(Ram::KEYBOARD_OFFSET, keys[i]);
        }
        let mut gamepad_map = 0;
        for i in 0..8 {
            if keys.contains(&self.context.peek(Ram::GAMEPAD_MAPPING_OFFSET + i)) {
                gamepad_map |= 1 << i;
            }
        }
        self.context.poke(Ram::GAMEPADS_OFFSET, gamepad_map);
        let mouse = wheel.get_mouse();
        let mouse_x = 
            if mouse.x > 0 && mouse.x <= 2 * (Vram::SCREEN_WIDTH + 2 * Self::BORDER_W) as i32 {
                mouse.x / 2 - Self::BORDER_W as i32
            } else {
                -1
            };
        let mouse_y = 
            if mouse.y > 0 && mouse.y <= 2 * (Vram::SCREEN_HEIGHT + 2 * Self::BORDER_H) as i32 {
                mouse.y / 2 - Self::BORDER_H as i32
            } else {
                -1
            };
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
            let y = (i + Self::BORDER_H) as i32 + y_offset;
            for xx in 0..Vram::SCREEN_WIDTH {
                let color_id = self.context.peek4(i * Vram::SCREEN_WIDTH + xx);
                if color_id != trans_color {
                    let color = palette[color_id as usize];
                    let x = (xx + Self::BORDER_W) as i32 + x_offset;
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
            let y = (i + Self::BORDER_H) as i32 + y_offset;
            for xx in 0..Vram::SCREEN_WIDTH {
                let color = palette[self.context.peek4(i * Vram::SCREEN_WIDTH + xx) as usize];
                let x = (xx + Self::BORDER_W) as i32 + x_offset;
                if !self.context.trans_map.contains(&(xx as i32, i as i32)) {
                    draw_fat_pixel(wheel, x, y, color);
                }
            }
            let color = self.get_color(self.context.peek(Vram::BORDER_COLOR_OFFSET));
            for x in 0..Self::BORDER_W as i32 {
                draw_fat_pixel(wheel, x, y, color);
                let x = x + Vram::SCREEN_WIDTH as i32;
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

        //todo: play sound
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
