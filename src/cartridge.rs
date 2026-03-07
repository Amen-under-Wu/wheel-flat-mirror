pub mod pix_mask;
pub mod ram;
use crate::{
    cartridge::{
        pix_mask::PixMask,
        ram::{Ram, Vram},
    },
    wheel_file::{ChunkType, WheelFile},
};
use std::{collections::HashMap, rc::Rc, cell::RefCell};

pub struct CartContext {
    pub ram: Ram,
    clip_rect: (i32, i32, i32, i32),
    pub key_timer: HashMap<u8, u32>,
    pub btn_timer: [i32; 32],
    ch_font: (Vec<u8>, Vec<u8>),
    pub file_data: Rc<RefCell<WheelFile>>,
}

impl CartContext {
    const BANK_N: usize = 8;
    pub fn new() -> Self {
        Self {
            ram: Ram::new(),
            clip_rect: (0, 0, Vram::SCREEN_WIDTH as i32, Vram::SCREEN_HEIGHT as i32),
            key_timer: HashMap::new(),
            btn_timer: [0; 32],
            ch_font: crate::data::ch_font(),
            file_data: Rc::new(RefCell::new(WheelFile::new())),
        }
    }

    pub fn get_subpix_map_mut(&mut self) -> &mut PixMask {
        self.ram.get_subpixels_mut()
    }

    // memory manipulations

    pub fn memcpy(&mut self, from: usize, to: usize, length: usize) {
        let buffer: Vec<u8> = (from..from + length)
            .into_iter()
            .map(|x| self.ram[x])
            .collect();
        for i in 0..length {
            self.ram[to + i] = buffer[i];
        }
    }
    pub fn memset(&mut self, addr: usize, value: u8, length: usize) {
        for i in 0..length {
            self.ram[addr + i] = value;
        }
    }
    pub fn peek_with_bits(&self, addr: usize, bits: usize) -> u8 {
        (self.ram[addr * bits / 8] >> (bits * (addr % (8 / bits)))) & ((1u16 << bits) - 1) as u8
    }
    pub fn peek(&self, addr: usize) -> u8 {
        self.ram[addr]
    }
    pub fn peek4(&self, addr: usize) -> u8 {
        (self.ram[addr / 2] >> (4 * (addr % 2))) & 0xf
    }
    pub fn peek2(&self, addr: usize) -> u8 {
        (self.ram[addr / 4] >> (2 * (addr % 4))) & 0b11
    }
    pub fn peek1(&self, addr: usize) -> u8 {
        (self.ram[addr / 8] >> (1 * (addr % 8))) & 0b1
    }
    pub fn poke_with_bits(&mut self, addr: usize, val: u8, bits: usize) {
        let byte_borrow = &mut self.ram[addr * bits / 8];
        let bit_offset = ((addr % (8 / bits)) * bits) as u8;
        let mask: u8 = ((((1u16 << bits) - 1) as u8) << bit_offset) ^ 0xff;
        let val_mask = val << bit_offset;
        *byte_borrow &= mask;
        *byte_borrow |= val_mask;
    }
    pub fn poke(&mut self, addr: usize, val: u8) {
        self.ram[addr] = val;
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
                res |= (self.ram[addr + i] as i32) << (i * 8);
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
                res |= (self.ram[addr + i] as i32) << (i * 8);
                self.ram[addr + i] = ((val >> (i * 8)) & 0xff) as u8;
            }
            res
        } else {
            0
        }
    }
    pub fn vbank(&mut self, id: usize) {
        if id < Vram::VBANK_N {
            self.ram.set_active_vbank(id);
        }
    }

    // graphics

    fn in_clip(&self, x: i32, y: i32) -> bool {
        x >= self.clip_rect.0
            && x < self.clip_rect.0 + self.clip_rect.2
            && y >= self.clip_rect.1
            && y < self.clip_rect.1 + self.clip_rect.3
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
                self.get_subpix_map_mut().del(x as usize, y as usize);
            }
        }
    }

    pub fn set_pix(&mut self, x: i32, y: i32, color: u8) {
        if self.in_clip(x, y) && color < 16 {
            self.poke4(y as usize * Vram::SCREEN_WIDTH + x as usize, color);
            self.get_subpix_map_mut().del(x as usize, y as usize);
        }
    }
    pub fn get_pix(&self, x: i32, y: i32) -> u8 {
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
                self.get_subpix_map_mut().del(xx as usize, yy as usize);
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
        let (x1, x2, y1, y2) = if x1 > x2 {
            (x2, x1, y2, y1)
        } else {
            (x1, x2, y1, y2)
        };
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
                    if -(dx * yi - dy * xi - dx * y1 + dy * x1)
                        > dx * yi - dy * (xi + 1.0) - dx * y1 + dy * x1
                    {
                        xi += 1.0;
                    }
                }
            } else {
                while xi <= x2 {
                    self.set_pix(xi as i32, yi as i32, color);
                    xi += 1.0;
                    if dx * yi - dy * xi - dx * y1 + dy * x1
                        > -(dx * (yi - 1.0) - dy * xi - dx * y1 + dy * x1)
                    {
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
                    if dx * yi - dy * xi - dx * y1 + dy * x1
                        > -(dx * yi - dy * (xi + 1.0) - dx * y1 + dy * x1)
                    {
                        xi += 1.0;
                    }
                }
            } else {
                while xi <= x2 {
                    self.set_pix(xi as i32, yi as i32, color);
                    xi += 1.0;
                    if -(dx * yi - dy * xi - dx * y1 + dy * x1)
                        > dx * (yi + 1.0) - dy * xi - dx * y1 + dy * x1
                    {
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
        for y in self.clip_rect.1..self.clip_rect.1 + self.clip_rect.3 {
            for x in self.clip_rect.0..self.clip_rect.0 + self.clip_rect.2 {
                self.poke4(y as usize * Vram::SCREEN_WIDTH + x as usize, color);
                self.get_subpix_map_mut().del(x as usize, y as usize);
            }
        }
    }

    fn putchar(
        &mut self,
        chr: u8,
        x: i32,
        y: i32,
        color: u8,
        is_fixed: bool,
        scale: i32,
        alt_font: bool,
    ) -> i32 {
        let chr: usize = (chr % 128).into(); // ascii characters only
        let font_offset = if alt_font {
            Ram::ALT_FONT_OFFSET
        } else {
            Ram::SYSTEM_FONT_OFFSET
        };
        if is_fixed {
            for i in 0..8 {
                let line_data = self.peek(font_offset + chr * 8 + i);
                for j in 0..8 {
                    if ((line_data >> j) & 1) != 0 {
                        self.rect(
                            x + j as i32 * scale,
                            y + i as i32 * scale,
                            scale,
                            scale,
                            color,
                        );
                    }
                }
            }
            self.peek(font_offset + Ram::FONT_PARAM_OFFSET_RELATIVE)
                .into()
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
                        self.rect(
                            x + j as i32 * scale,
                            y + i as i32 * scale,
                            scale,
                            scale,
                            color,
                        );
                    }
                }
            }
            scale
                * (if zero_flag {
                    self.peek(font_offset + Ram::FONT_PARAM_OFFSET_RELATIVE) as i32 - 2
                } else {
                    max_bit - min_bit + 2
                })
        }
    }
    pub fn print(
        &mut self,
        text: &str,
        x: i32,
        y: i32,
        color: u8,
        is_fixed: bool,
        scale: i32,
        alt_font: bool,
    ) -> i32 {
        let mut text_width = 0;
        let mut x_offset = 0;
        let mut y = y;
        let chr_height: i32 = scale
            * self.peek(
                (if alt_font {
                    Ram::ALT_FONT_OFFSET
                } else {
                    Ram::SYSTEM_FONT_OFFSET
                }) + Ram::FONT_PARAM_OFFSET_RELATIVE
                    + 1,
            ) as i32;
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

    const fn spr_pix_arr() -> [(usize, usize); Ram::SPRITE_W * Ram::SPRITE_H] {
        let mut arr = [(0, 0); Ram::SPRITE_W * Ram::SPRITE_H];
        let mut i = 0;
        while i < Ram::SPRITE_W * Ram::SPRITE_H {
            let y = i / Ram::SPRITE_W;
            let x = i % Ram::SPRITE_W;
            arr[i] = (x, y);
            i += 1;
        }
        arr
    }
    const SPR_PIX_ARR: [(usize, usize); Ram::SPRITE_W * Ram::SPRITE_H] = Self::spr_pix_arr();
    fn get_spr_pix(&self, id: usize, x: usize, y: usize, bpp: usize) -> u8 {
        self.peek_with_bits(
            Ram::TILES_OFFSET * 8 / bpp
                + id * Ram::SPRITE_W * Ram::SPRITE_H
                + y * Ram::SPRITE_W
                + x,
            bpp,
        )
    }
    fn spr_mono_unchecked(
        &mut self,
        id: usize,
        x: i32,
        y: i32,
        trans_color: u8,
        scale: i32,
        flip: u8,
        rotate: u8,
        bpp: usize,
    ) {
        match (rotate << 2) + flip {
            0 | 11 => {
                for (i, j) in Self::SPR_PIX_ARR {
                    let color = self.get_spr_pix(id, j, i, bpp);
                    if color != trans_color {
                        self.rect(
                            x + j as i32 * scale,
                            y + i as i32 * scale,
                            scale,
                            scale,
                            color,
                        );
                    }
                }
            }
            1 | 10 => {
                for (i, j) in Self::SPR_PIX_ARR {
                    let color = self.get_spr_pix(id, Ram::SPRITE_W - j - 1, i, bpp);
                    if color != trans_color {
                        self.rect(
                            x + j as i32 * scale,
                            y + i as i32 * scale,
                            scale,
                            scale,
                            color,
                        );
                    }
                }
            }
            2 | 9 => {
                for (i, j) in Self::SPR_PIX_ARR {
                    let color = self.get_spr_pix(id, j, Ram::SPRITE_H - i - 1, bpp);
                    if color != trans_color {
                        self.rect(
                            x + j as i32 * scale,
                            y + i as i32 * scale,
                            scale,
                            scale,
                            color,
                        );
                    }
                }
            }
            3 | 8 => {
                for (i, j) in Self::SPR_PIX_ARR {
                    let color =
                        self.get_spr_pix(id, Ram::SPRITE_W - j - 1, Ram::SPRITE_H - i - 1, bpp);
                    if color != trans_color {
                        self.rect(
                            x + j as i32 * scale,
                            y + i as i32 * scale,
                            scale,
                            scale,
                            color,
                        );
                    }
                }
            }
            4 | 15 => {
                for (i, j) in Self::SPR_PIX_ARR {
                    let color = self.get_spr_pix(id, Ram::SPRITE_W - j - 1, i, bpp);
                    if color != trans_color {
                        self.rect(
                            x + i as i32 * scale,
                            y + j as i32 * scale,
                            scale,
                            scale,
                            color,
                        );
                    }
                }
            }
            5 | 14 => {
                for (i, j) in Self::SPR_PIX_ARR {
                    let color =
                        self.get_spr_pix(id, Ram::SPRITE_W - j - 1, Ram::SPRITE_H - i - 1, bpp);
                    if color != trans_color {
                        self.rect(
                            x + i as i32 * scale,
                            y + j as i32 * scale,
                            scale,
                            scale,
                            color,
                        );
                    }
                }
            }
            6 | 13 => {
                for (i, j) in Self::SPR_PIX_ARR {
                    let color = self.get_spr_pix(id, j, i, bpp);
                    if color != trans_color {
                        self.rect(
                            x + i as i32 * scale,
                            y + j as i32 * scale,
                            scale,
                            scale,
                            color,
                        );
                    }
                }
            }
            7 | 12 => {
                for (i, j) in Self::SPR_PIX_ARR {
                    let color = self.get_spr_pix(id, j, Ram::SPRITE_H - i - 1, bpp);
                    if color != trans_color {
                        self.rect(
                            x + i as i32 * scale,
                            y + j as i32 * scale,
                            scale,
                            scale,
                            color,
                        );
                    }
                }
            }
            _ => (),
        }
    }
    pub fn spr(
        &mut self,
        id: i32,
        x: i32,
        y: i32,
        trans_color: u8,
        scale: i32,
        flip: i32,
        rotate: i32,
        w: i32,
        h: i32,
    ) {
        const N: i32 = (Ram::CANVAS_W * Ram::CANVAS_H) as i32;
        if id < 0 || id > N * 2 || w <= 0 || h <= 0 {
            return;
        }
        let id = id as usize;
        let flip = (flip.clamp(0, 4) % 4) as u8;
        let rotate = (rotate.clamp(0, 4) % 4) as u8;
        let w = w.min((Ram::CANVAS_W - id % Ram::CANVAS_W) as i32);
        let bpp = 8 / self.peek4(Vram::BLIT_SEGMENT_OFFSET * 2) as usize;
        for i in 0..h {
            if (id as i32 % N) / Ram::CANVAS_W as i32 + i > Ram::CANVAS_H as i32 {
                break;
            }
            for j in 0..w {
                match (rotate << 2) | flip {
                    0 | 11 => self.spr_mono_unchecked(
                        id + i as usize * Ram::CANVAS_W + j as usize,
                        x + j * Ram::SPRITE_W as i32 * scale,
                        y + i * Ram::SPRITE_H as i32 * scale,
                        trans_color,
                        scale,
                        flip,
                        rotate,
                        bpp,
                    ),
                    1 | 10 => self.spr_mono_unchecked(
                        id + i as usize * Ram::CANVAS_W + j as usize,
                        x + (w - j - 1) * Ram::SPRITE_W as i32 * scale,
                        y + i * Ram::SPRITE_H as i32 * scale,
                        trans_color,
                        scale,
                        flip,
                        rotate,
                        bpp,
                    ),
                    2 | 9 => self.spr_mono_unchecked(
                        id + i as usize * Ram::CANVAS_W + j as usize,
                        x + j * Ram::SPRITE_W as i32 * scale,
                        y + (h - i - 1) * Ram::SPRITE_H as i32 * scale,
                        trans_color,
                        scale,
                        flip,
                        rotate,
                        bpp,
                    ),
                    3 | 8 => self.spr_mono_unchecked(
                        id + i as usize * Ram::CANVAS_W + j as usize,
                        x + (w - j - 1) * Ram::SPRITE_W as i32 * scale,
                        y + (h - i - 1) * Ram::SPRITE_H as i32 * scale,
                        trans_color,
                        scale,
                        flip,
                        rotate,
                        bpp,
                    ),
                    4 | 15 => self.spr_mono_unchecked(
                        id + i as usize * Ram::CANVAS_W + j as usize,
                        x + i * Ram::SPRITE_W as i32 * scale,
                        y + (w - j - 1) * Ram::SPRITE_H as i32 * scale,
                        trans_color,
                        scale,
                        flip,
                        rotate,
                        bpp,
                    ),
                    5 | 14 => self.spr_mono_unchecked(
                        id + i as usize * Ram::CANVAS_W + j as usize,
                        x + (h - i - 1) * Ram::SPRITE_W as i32 * scale,
                        y + (w - j - 1) * Ram::SPRITE_H as i32 * scale,
                        trans_color,
                        scale,
                        flip,
                        rotate,
                        bpp,
                    ),
                    6 | 13 => self.spr_mono_unchecked(
                        id + i as usize * Ram::CANVAS_W + j as usize,
                        x + i * Ram::SPRITE_W as i32 * scale,
                        y + j * Ram::SPRITE_H as i32 * scale,
                        trans_color,
                        scale,
                        flip,
                        rotate,
                        bpp,
                    ),
                    7 | 12 => self.spr_mono_unchecked(
                        id + i as usize * Ram::CANVAS_W + j as usize,
                        x + (h - i - 1) * Ram::SPRITE_W as i32 * scale,
                        y + j * Ram::SPRITE_H as i32 * scale,
                        trans_color,
                        scale,
                        flip,
                        rotate,
                        bpp,
                    ),
                    _ => (),
                }
            }
        }
    }
    pub fn fget(&self, id: i32, flag_index: i32) -> bool {
        if id >= 0 && id < 512 && flag_index >= 0 && flag_index < 8 {
            self.peek1(Ram::SPRITE_FLAGS_OFFSET * 8 + id as usize * 8 + flag_index as usize) == 1
        } else {
            false
        }
    }
    pub fn fset(&mut self, id: i32, flag_index: i32, value: bool) {
        if id >= 0 && id < 512 && flag_index >= 0 && flag_index < 8 {
            self.poke1(
                Ram::SPRITE_FLAGS_OFFSET * 8 + id as usize * 8 + flag_index as usize,
                if value { 1 } else { 0 },
            );
        }
    }

    pub fn mget(&self, x: i32, y: i32) -> i32 {
        if x >= 0 && y >= 0 && x < Ram::MAP_W as i32 && y < Ram::MAP_H as i32 {
            self.peek(Ram::MAP_OFFSET + y as usize * Ram::MAP_W + x as usize)
                .into()
        } else {
            -1
        }
    }
    pub fn mset(&mut self, x: i32, y: i32, tile_id: u8) {
        if x >= 0 && y >= 0 && x < Ram::MAP_W as i32 && y < Ram::MAP_H as i32 {
            self.poke(
                Ram::MAP_OFFSET + y as usize * Ram::MAP_W + x as usize,
                tile_id,
            );
        }
    }
    pub fn map(
        &mut self,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        sx: i32,
        sy: i32,
        trans_color: u8,
        scale: i32,
    ) {
        // remap param to be added when wrapping to lua, not here
        // upd: adding remap requires rewriting the whole function in script lang. keep it in mind.
        for i in 0..h {
            for j in 0..w {
                self.spr(
                    self.mget(sx + j, sy + i),
                    x + j * Ram::TILE_W as i32 * scale,
                    y + i * Ram::TILE_H as i32 * scale,
                    trans_color,
                    scale,
                    0,
                    0,
                    1,
                    1,
                );
            } // maybe too much boundary checks?
        }
    }
    pub fn map_with_remap(
        &mut self,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        sx: i32,
        sy: i32,
        trans_color: u8,
        scale: i32,
        mut remap: Box<dyn FnMut(i32, i32, i32) -> (i32, i32, i32)>,
    ) {
        for i in 0..h {
            for j in 0..w {
                let (id, flip, rotate) = remap(self.mget(sx + j, sy + i), sx + j, sy + i);
                self.spr(
                    id,
                    x + j * Ram::TILE_W as i32 * scale,
                    y + i * Ram::TILE_H as i32 * scale,
                    trans_color,
                    scale,
                    flip,
                    rotate,
                    1,
                    1,
                );
            }
        }
    }

    fn cross_mult(x1: f32, y1: f32, x2: f32, y2: f32, x0: f32, y0: f32) -> f32 {
        (x2 - x1) * (y0 - y1) - (y2 - y1) * (x0 - x1)
    }

    pub fn tri(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x3: f32, y3: f32, color: u8) {
        let x_max = (x1.ceil().max(x2.ceil()).max(x3.ceil()) as i32)
            .min(self.clip_rect.0 + self.clip_rect.2 - 1);
        let x_min = (x1.floor().min(x2.floor()).min(x3.floor()) as i32).max(self.clip_rect.0);
        let y_max = (y1.ceil().max(y2.ceil()).max(y3.ceil()) as i32)
            .min(self.clip_rect.1 + self.clip_rect.3 - 1);
        let y_min = (y1.floor().min(y2.floor()).min(y3.floor()) as i32).max(self.clip_rect.1);
        for y in y_min..=y_max {
            for x in x_min..=x_max {
                let cross1 = Self::cross_mult(x1, y1, x2, y2, x as f32, y as f32);
                let cross2 = Self::cross_mult(x2, y2, x3, y3, x as f32, y as f32);
                let cross3 = Self::cross_mult(x3, y3, x1, y1, x as f32, y as f32);
                if (cross1 >= 0.0 && cross2 >= 0.0 && cross3 >= 0.0)
                    || (cross1 <= 0.0 && cross2 <= 0.0 && cross3 <= 0.0)
                {
                    self.set_pix(x, y, color);
                }
            }
        }
    }
    pub fn trib(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x3: f32, y3: f32, color: u8) {
        self.line(x1, y1, x2, y2, color);
        self.line(x2, y2, x3, y3, color);
        self.line(x3, y3, x1, y1, color);
    }

    // the following 2 functions are adapted from cpp macros;
    // therefore they look messy
    // original macros written in 2024.6.
    fn get_tile_pix(&self, x: i32, y: i32) -> u8 {
        if x >= 0
            && y >= 0
            && x < (Ram::CANVAS_W * Ram::TILE_W) as i32
            && y < (Ram::CANVAS_H * Ram::TILE_H) as i32
        {
            let x = x as usize;
            let y = y as usize;
            self.peek4(
                Ram::TILES_OFFSET * 2
                    + (y / Ram::TILE_H * Ram::CANVAS_W + x / Ram::TILE_W) * Ram::TILE_BYTE_SIZE * 2
                    + (y % Ram::TILE_H * Ram::TILE_W + x % Ram::TILE_W),
            )
        } else {
            255
        }
    }
    fn get_map_pix(&self, x: i32, y: i32) -> u8 {
        if x >= 0
            && y >= 0
            && x < (Ram::MAP_W * Ram::TILE_W) as i32
            && y < (Ram::MAP_H * Ram::TILE_H) as i32
        {
            let x = x as usize;
            let y = y as usize;
            self.peek4(
                Ram::TILES_OFFSET * 2
                    + self.peek(
                        Ram::MAP_OFFSET
                            + ((y % (Ram::TILE_H * Ram::MAP_H) + (Ram::TILE_H * Ram::MAP_H))
                                % (Ram::TILE_H * Ram::MAP_H))
                                / Ram::TILE_H
                                * Ram::MAP_W
                            + (((x) % (Ram::MAP_W * Ram::TILE_W) + (Ram::MAP_W * Ram::TILE_W))
                                % (Ram::MAP_W * Ram::TILE_W))
                                / Ram::TILE_W,
                    ) as usize
                        * Ram::TILE_BYTE_SIZE
                        * 2
                    + (((y) % Ram::TILE_H + Ram::TILE_H) % Ram::TILE_H) * Ram::TILE_W
                    + (((x) % Ram::TILE_W + Ram::TILE_W) % Ram::TILE_W),
            )
        } else {
            255
        }
    }
    pub fn textri(
        &mut self,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        x3: f32,
        y3: f32,
        u1: f32,
        v1: f32,
        u2: f32,
        v2: f32,
        u3: f32,
        v3: f32,
        use_map: bool,
        trans_color: u8,
    ) {
        let x_max = (x1.ceil().max(x2.ceil()).max(x3.ceil()) as i32)
            .min(self.clip_rect.0 + self.clip_rect.2 - 1);
        let x_min = (x1.floor().min(x2.floor()).min(x3.floor()) as i32).max(self.clip_rect.0);
        let y_max = (y1.ceil().max(y2.ceil()).max(y3.ceil()) as i32)
            .min(self.clip_rect.1 + self.clip_rect.3 - 1);
        let y_min = (y1.floor().min(y2.floor()).min(y3.floor()) as i32).max(self.clip_rect.1);
        let mat_inv: [[f32; 3]; 3] = [
            [y3 - y2, y1 - y3, y2 - y1],
            [x2 - x3, x3 - x1, x1 - x2],
            [x3 * y2 - x2 * y3, x1 * y3 - x3 * y1, x2 * y1 - x1 * y2],
        ];
        let dnm = x3 * y2 - x2 * y3 + x1 * y3 - x3 * y1 + x2 * y1 - x1 * y2;
        let a = (u1 * mat_inv[0][0] + u2 * mat_inv[0][1] + u3 * mat_inv[0][2]) / dnm;
        let b = (u1 * mat_inv[1][0] + u2 * mat_inv[1][1] + u3 * mat_inv[1][2]) / dnm;
        let c = (u1 * mat_inv[2][0] + u2 * mat_inv[2][1] + u3 * mat_inv[2][2]) / dnm;
        let d = (v1 * mat_inv[0][0] + v2 * mat_inv[0][1] + v3 * mat_inv[0][2]) / dnm;
        let e = (v1 * mat_inv[1][0] + v2 * mat_inv[1][1] + v3 * mat_inv[1][2]) / dnm;
        let f = (v1 * mat_inv[2][0] + v2 * mat_inv[2][1] + v3 * mat_inv[2][2]) / dnm;
        if use_map {
            for y in y_min..=y_max {
                let y = y as f32;
                for x in x_min..=x_max {
                    let x = x as f32;
                    let cross1 = Self::cross_mult(x1, y1, x2, y2, x, y);
                    let cross2 = Self::cross_mult(x2, y2, x3, y3, x, y);
                    let cross3 = Self::cross_mult(x3, y3, x1, y1, x, y);
                    if (cross1 >= 0.0 && cross2 >= 0.0 && cross3 >= 0.0)
                        || (cross1 <= 0.0 && cross2 <= 0.0 && cross3 <= 0.0)
                    {
                        let u = (a * x + b * y + c).round() as i32;
                        let v = (d * x + e * y + f).round() as i32;
                        let color = self.get_map_pix(u, v);
                        if color != trans_color {
                            self.set_pix(x as i32, y as i32, color);
                        }
                    }
                }
            }
        } else {
            for y in y_min..=y_max {
                let y = y as f32;
                for x in x_min..=x_max {
                    let x = x as f32;
                    let cross1 = Self::cross_mult(x1, y1, x2, y2, x, y);
                    let cross2 = Self::cross_mult(x2, y2, x3, y3, x, y);
                    let cross3 = Self::cross_mult(x3, y3, x1, y1, x, y);
                    if (cross1 >= 0.0 && cross2 >= 0.0 && cross3 >= 0.0)
                        || (cross1 <= 0.0 && cross2 <= 0.0 && cross3 <= 0.0)
                    {
                        let u = (a * x + b * y + c).round() as i32;
                        let v = (d * x + e * y + f).round() as i32;
                        let color = self.get_tile_pix(u, v);
                        if color != trans_color {
                            self.set_pix(x as i32, y as i32, color);
                        }
                    }
                }
            }
        }
    }

    fn subpix_2_pix(x: i32, y: i32) -> (i32, i32, usize) {
        (x / 2, y / 2, ((y % 2) * 2 + x % 2) as usize)
    }
    fn putchar_ch_7px(&mut self, chr: char, x: i32, y: i32, color: u8, scale: i32) -> i32 {
        // better without upscaling
        let offset = (chr as usize - '一' as usize) * 8;
        for i in 0..8 {
            let line_data = self.ch_font.1[offset + i];
            for j in 0..8 {
                if ((line_data >> j) & 1) != 0 {
                    self.rect(
                        x + j as i32 * scale,
                        y + i as i32 * scale,
                        scale,
                        scale,
                        color,
                    );
                }
            }
        }
        scale * 8
    }
    fn putchar_ch_16px(&mut self, chr: char, x: i32, y: i32, color: u8, scale: i32) -> i32 {
        let offset = (chr as usize - '一' as usize) * 32;
        for i in 0..16 {
            let line_data = self.ch_font.0[offset + i * 2];
            for j in 0..8 {
                if ((line_data >> j) & 1) != 0 {
                    for k in 0..scale {
                        for l in 0..scale {
                            let pix = Self::subpix_2_pix(
                                x * 2 + j as i32 * scale + k,
                                y * 2 + i as i32 * scale + l,
                            );
                            if self.in_clip(pix.0, pix.1) {
                                self.get_subpix_map_mut().set(
                                    pix.0 as usize,
                                    pix.1 as usize,
                                    pix.2,
                                    color,
                                );
                            }
                        }
                    }
                }
            }
            let line_data = self.ch_font.0[offset + i * 2 + 1];
            for j in 0..8 {
                if ((line_data >> j) & 1) != 0 {
                    for k in 0..scale {
                        for l in 0..scale {
                            let pix = Self::subpix_2_pix(
                                x * 2 + (j + 8) as i32 * scale + k,
                                y * 2 + i as i32 * scale + l,
                            );
                            if self.in_clip(pix.0, pix.1) {
                                self.get_subpix_map_mut().set(
                                    pix.0 as usize,
                                    pix.1 as usize,
                                    pix.2,
                                    color,
                                );
                            }
                        }
                    }
                }
            }
        }
        scale * 8 + 1
    }
    pub fn print_ch(
        &mut self,
        text: &str,
        x: i32,
        y: i32,
        color: u8,
        is_fixed: bool,
        scale: i32,
        alt_font: bool,
    ) -> i32 {
        let mut text_width = 0;
        let mut x_offset = 0;
        let mut y = y;
        let chr_height_en: i32 = scale
            * self.peek(
                (if alt_font {
                    Ram::ALT_FONT_OFFSET
                } else {
                    Ram::SYSTEM_FONT_OFFSET
                }) + Ram::FONT_PARAM_OFFSET_RELATIVE
                    + 1,
            ) as i32;
        let chr_height: i32 = scale * 8;
        for chr in text.chars() {
            let chr = chr as u32;
            if chr == '\n' as u32 {
                text_width = text_width.max(x_offset);
                x_offset = 0;
                y += chr_height;
            } else {
                if chr >= '一' as u32 && chr <= '龥' as u32 {
                    if alt_font {
                        x_offset += self.putchar_ch_7px(
                            char::from_u32(chr).unwrap_or('㗊'),
                            x + x_offset,
                            y,
                            color,
                            scale,
                        );
                    } else {
                        x_offset += self.putchar_ch_16px(
                            char::from_u32(chr).unwrap_or('㗊'),
                            x + x_offset,
                            y,
                            color,
                            scale,
                        );
                    }
                } else {
                    let y = y - (chr_height_en - chr_height);
                    x_offset += self.putchar(
                        chr.clamp(0, 128) as u8,
                        x + x_offset,
                        y,
                        color,
                        is_fixed,
                        scale,
                        alt_font,
                    );
                }
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
        let key_buffer: Vec<u8> = (0..4)
            .into_iter()
            .map(|i| self.peek(Ram::KEYBOARD_OFFSET + i))
            .collect();
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
            Some(&time) => time == 0 || (time > hold && (time - hold) % period == 0),
        }
    }
    pub fn mouse(&self) -> (u8, u8, bool, bool, bool, i8, i8) {
        let x = self.peek(Ram::MOUSE_OFFSET) as u8;
        let y = self.peek(Ram::MOUSE_OFFSET + 1) as u8;
        let res: u16 = self.peek(Ram::MOUSE_OFFSET + 2) as u16
            | ((self.peek(Ram::MOUSE_OFFSET + 3) as u16) << 8);
        (
            x,
            y,
            (res & 1) != 0,
            (res & 2) != 0,
            (res & 4) != 0,
            ((res >> 1) as i8) >> 2,
            ((res >> 7) as i8) >> 2,
        )
    }

    fn load_from_cart(&mut self, mask: u8, bank: u8) {
        if (mask & 1) != 0 {
            if let Some(data) = self.file_data.borrow().get_chunk(ChunkType::Tiles, bank) {
                for i in 0..data.data.len().min(Ram::TILES_BYTE_SIZE) {
                    self.ram[Ram::TILES_OFFSET + i] = data.data[i];
                }
            }
        }
        if (mask & 2) != 0 {
            if let Some(data) = self.file_data.borrow().get_chunk(ChunkType::Sprites, bank) {
                for i in 0..data.data.len().min(Ram::SPRITES_BYTE_SIZE) {
                    self.ram[Ram::SPRITES_OFFSET + i] = data.data[i];
                }
            }
        }
        if (mask & 4) != 0 {
            if let Some(data) = self.file_data.borrow().get_chunk(ChunkType::Map, bank) {
                for i in 0..data.data.len().min(Ram::MAP_BYTE_SIZE) {
                    self.ram[Ram::MAP_OFFSET + i] = data.data[i];
                }
            }
        }
        if (mask & 8) != 0 {
            if let Some(data) = self.file_data.borrow().get_chunk(ChunkType::Sfx, bank) {
                for i in 0..data.data.len().min(Ram::SFX_BYTE_SIZE) {
                    self.ram[Ram::SFX_OFFSET + i] = data.data[i];
                }
            }
        }
        if (mask & 16) != 0 {
            if let Some(data) = self.file_data.borrow().get_chunk(ChunkType::Music, bank) {
                for i in 0..data.data.len().min(Ram::MUSIC_TRACKS_BYTE_SIZE) {
                    self.ram[Ram::MUSIC_TRACKS_OFFSET + i] = data.data[i];
                }
            }
            if let Some(data) = self.file_data.borrow().get_chunk(ChunkType::Patterns, bank) {
                for i in 0..data.data.len().min(Ram::MUSIC_PATTERNS_BYTE_SIZE) {
                    self.ram[Ram::MUSIC_PATTERNS_OFFSET + i] = data.data[i];
                }
            }
        }
        if (mask & 32) != 0 {
            if let Some(data) = self.file_data.borrow().get_chunk(ChunkType::Palette, bank) {
                for i in 0..data.data.len().min(Vram::PALETTE_BYTE_SIZE) {
                    self.ram[Vram::PALETTE_OFFSET + i] = data.data[i];
                }
            }
        }
        if (mask & 64) != 0 {
            if let Some(data) = self.file_data.borrow().get_chunk(ChunkType::Flags, bank) {
                for i in 0..data.data.len().min(Ram::SPRITE_FLAGS_BYTE_SIZE) {
                    self.ram[Ram::SPRITE_FLAGS_OFFSET + i] = data.data[i];
                }
            }
        }
        if (mask & 128) != 0 {
            if let Some(data) = self.file_data.borrow().get_chunk(ChunkType::Screen, bank) {
                for i in 0..data.data.len().min(Vram::SCREEN_BYTE_SIZE) {
                    self.ram[i] = data.data[i];
                }
            }
        }
    }
    fn save_to_cart(&mut self, mask: u8, bank: u8) {
        if (mask & 1) != 0 {
            let data = (0..Ram::TILES_BYTE_SIZE)
                .map(|i| self.ram[Ram::TILES_OFFSET + i])
                .collect();
            self.file_data.borrow_mut().set_chunk(ChunkType::Tiles, bank, data);
        }
        if (mask & 2) != 0 {
            let data = (0..Ram::SPRITES_BYTE_SIZE)
                .map(|i| self.ram[Ram::SPRITES_OFFSET + i])
                .collect();
            self.file_data.borrow_mut().set_chunk(ChunkType::Sprites, bank, data);
        }
        if (mask & 4) != 0 {
            let data = (0..Ram::MAP_BYTE_SIZE)
                .map(|i| self.ram[Ram::MAP_OFFSET + i])
                .collect();
            self.file_data.borrow_mut().set_chunk(ChunkType::Map, bank, data);
        }
        if (mask & 8) != 0 {
            let data = (0..Ram::SFX_BYTE_SIZE)
                .map(|i| self.ram[Ram::SFX_OFFSET + i])
                .collect();
            self.file_data.borrow_mut().set_chunk(ChunkType::Sfx, bank, data);
        }
        if (mask & 16) != 0 {
            let data = (0..Ram::MUSIC_TRACKS_BYTE_SIZE)
                .map(|i| self.ram[Ram::MUSIC_TRACKS_OFFSET + i])
                .collect();
            self.file_data.borrow_mut().set_chunk(ChunkType::Music, bank, data);
            let data = (0..Ram::MUSIC_PATTERNS_BYTE_SIZE)
                .map(|i| self.ram[Ram::MUSIC_PATTERNS_OFFSET + i])
                .collect();
            self.file_data.borrow_mut().set_chunk(ChunkType::Patterns, bank, data);
        }
        if (mask & 32) != 0 {
            let data = (0..Vram::PALETTE_BYTE_SIZE)
                .map(|i| self.ram[Vram::PALETTE_OFFSET + i])
                .collect();
            self.file_data.borrow_mut().set_chunk(ChunkType::Palette, bank, data);
        }
        if (mask & 64) != 0 {
            let data = (0..Ram::SPRITE_FLAGS_BYTE_SIZE)
                .map(|i| self.ram[Ram::SPRITE_FLAGS_OFFSET + i])
                .collect();
            self.file_data.borrow_mut().set_chunk(ChunkType::Flags, bank, data);
        }
        if (mask & 128) != 0 {
            let data = (0..Vram::SCREEN_BYTE_SIZE).map(|i| self.ram[i]).collect();
            self.file_data.borrow_mut().set_chunk(ChunkType::Screen, bank, data);
        }
    }
    pub fn sync(&mut self, mask: u8, bank: u8, to_cart: bool) {
        let mask = if mask == 0 { 0b11111111 } else { mask };
        if to_cart {
            self.save_to_cart(mask, bank);
        } else {
            self.load_from_cart(mask, bank);
        }
    }
    pub fn set_file_ptr(&mut self, file_ptr: Rc<RefCell<WheelFile>>) {
        self.file_data = file_ptr;
    }
}

#[cfg(test)]
mod tests {
    use crate::cartridge::CartContext;
    use crate::cartridge::ram::{Ram, Vram};
    #[test]
    fn test_ram() {
        let mut context = CartContext::new();
        assert_eq!(Vram::BLIT_SEGMENT_OFFSET, 0x3ffc);
        assert_eq!(Ram::GAMEPAD_MAPPING_OFFSET, 0x14e04);
        assert_eq!(Ram::GAMEPADS_OFFSET, 0xff80);
        //context.poke(0x12345, 0xab);
        context.poke_with_bits(0x12345 * 2, 0xb, 4);
        context.poke_with_bits(0x12345 * 2 + 1, 0xa, 4);
        assert_eq!(
            context.peek4(0x12345 * 2),
            context.peek_with_bits(0x12345 * 2, 4)
        );
        assert_eq!(context.peek4(0x12345 * 2), 0xb);
        assert_eq!(context.peek4(0x12345 * 2 + 1), 0xa);
        context.set_pmem(12, 0x01234567);
        assert_eq!(context.get_pmem(12), 0x01234567);
        assert_eq!(
            context.peek(Ram::PERSISTENT_MEMORY_OFFSET + 12 * 4 + 1),
            0x45
        );
        context.set_pmem(12, 0xfedc9999u32 as i32);
        assert_eq!(context.get_pmem(12), 0xfedc9999u32 as i32);
    }
}
