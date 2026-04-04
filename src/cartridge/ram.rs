use crate::cartridge::pix_mask::PixMask;

pub struct Vram {
    subpixels: PixMask,
}

impl Vram {
    pub const SIZE: usize = 16384;
    pub const VBANK_N: usize = 2;
    pub const SCREEN_WIDTH: usize = 240;
    pub const SCREEN_HEIGHT: usize = 136;
    const BPP: usize = 4; // bits per pixel
    pub const SCREEN_BYTE_SIZE: usize = Self::SCREEN_WIDTH * Self::SCREEN_HEIGHT * Self::BPP / 8;
    pub const PALETTE_OFFSET: usize = Self::SCREEN_BYTE_SIZE;
    pub const PALETTE_SIZE: usize = 16;
    pub const PALETTE_BYTE_SIZE: usize = Self::PALETTE_SIZE * 3;
    pub const PALETTE_MAP_OFFSET: usize = Self::PALETTE_OFFSET + Self::PALETTE_BYTE_SIZE;
    const PALETTE_MAP_BYTE_SIZE: usize =
        Self::PALETTE_SIZE * Self::PALETTE_SIZE.ilog2() as usize / 8;
    pub const BORDER_COLOR_OFFSET: usize = Self::PALETTE_MAP_OFFSET + Self::PALETTE_MAP_BYTE_SIZE;
    const BORDER_COLOR_BYTE_SIZE: usize = 1;
    pub const SCREEN_OFFSET_OFFSET: usize =
        Self::BORDER_COLOR_OFFSET + Self::BORDER_COLOR_BYTE_SIZE;
    const SCREEN_OFFSET_BYTE_SIZE: usize = 2;
    pub const MOUSE_CURSOR_OFFSET: usize =
        Self::SCREEN_OFFSET_OFFSET + Self::SCREEN_OFFSET_BYTE_SIZE;
    const MOUSE_CURSOR_BYTE_SIZE: usize = 1;
    pub const BLIT_SEGMENT_OFFSET: usize = Self::MOUSE_CURSOR_OFFSET + Self::MOUSE_CURSOR_BYTE_SIZE;

    pub fn new() -> Self {
        Self {
            subpixels: PixMask::new(Self::SCREEN_WIDTH * Self::SCREEN_HEIGHT),
        }
    }
}

pub struct Ram {
    pub vram: Vram,
    pub ram: [u8; Self::SIZE + Vram::SIZE],
    pub active_vbank: usize,
}

impl Ram {
    pub const SIZE: usize = 0x18000;
    pub const TILES_OFFSET: usize = Vram::SIZE;
    const BPP: usize = 4;
    pub const CANVAS_W: usize = 16;
    pub const CANVAS_H: usize = 16;
    const TILES_N: usize = Self::CANVAS_W * Self::CANVAS_H;
    pub const TILE_W: usize = 8;
    pub const TILE_H: usize = 8;
    pub const TILE_BYTE_SIZE: usize = Self::TILE_W * Self::TILE_H * Self::BPP / 8;
    pub const TILES_BYTE_SIZE: usize = Self::TILES_N * Self::TILE_BYTE_SIZE;
    pub const SPRITES_OFFSET: usize = Self::TILES_OFFSET + Self::TILES_BYTE_SIZE;
    const SPRITES_N: usize = Self::TILES_N;
    pub const SPRITE_W: usize = 8;
    pub const SPRITE_H: usize = 8;
    pub const SPRITE_BYTE_SIZE: usize = Self::SPRITE_W * Self::SPRITE_H * Self::BPP / 8;
    pub const SPRITES_BYTE_SIZE: usize = Self::SPRITES_N * Self::SPRITE_BYTE_SIZE;
    pub const MAP_OFFSET: usize = Self::SPRITES_OFFSET + Self::SPRITES_BYTE_SIZE;
    pub const MAP_W: usize = 240;
    pub const MAP_H: usize = 136;
    const MAP_TILE_BYTE_SIZE: usize = 1;
    pub const MAP_BYTE_SIZE: usize = Self::MAP_W * Self::MAP_H * Self::MAP_TILE_BYTE_SIZE;
    pub const GAMEPADS_OFFSET: usize = Self::MAP_OFFSET + Self::MAP_BYTE_SIZE;
    const GAMEPADS_BYTE_SIZE: usize = 4;
    pub const MOUSE_OFFSET: usize = Self::GAMEPADS_OFFSET + Self::GAMEPADS_BYTE_SIZE;
    const MOUSE_BYTE_SIZE: usize = 4;
    pub const KEYBOARD_OFFSET: usize = Self::MOUSE_OFFSET + Self::MOUSE_BYTE_SIZE;
    const KEYBOARD_BYTE_SIZE: usize = 4;
    pub const SFX_STATE_OFFSET: usize = Self::KEYBOARD_OFFSET + Self::KEYBOARD_BYTE_SIZE;
    const SFX_STATE_BYTE_SIZE: usize = 16;
    pub const SOUND_REGISTERS_OFFSET: usize = Self::SFX_STATE_OFFSET + Self::SFX_STATE_BYTE_SIZE;
    pub const SOUND_REGISTER_SIZE: usize = 18;
    const SOUND_REGISTERS_BYTE_SIZE: usize = Self::SOUND_REGISTER_SIZE * 4;
    pub const WAVEFORMS_OFFSET: usize =
        Self::SOUND_REGISTERS_OFFSET + Self::SOUND_REGISTERS_BYTE_SIZE;
    const WAVEFORMS_N: usize = 16;
    const WAVEFORM_SAMPLE_N: usize = 32;
    const WAVEFORM_BPS: usize = 4; // bits per sample
    pub const WAVEFORMS_BYTE_SIZE: usize =
        Self::WAVEFORMS_N * Self::WAVEFORM_SAMPLE_N * Self::WAVEFORM_BPS / 8;
    pub const SFX_OFFSET: usize = Self::WAVEFORMS_OFFSET + Self::WAVEFORMS_BYTE_SIZE;
    pub const SFX_N: usize = 64;
    pub const SFX_FRAME_N: usize = 30;
    pub const SFX_FRAME_BYTE_SIZE: usize = 2;
    pub const SFX_DATA_OFFSET_SELF: usize = Self::SFX_FRAME_BYTE_SIZE * Self::SFX_FRAME_N;
    pub const SFX_DATA_BYTE_SIZE: usize = 2;
    pub const SFX_LOOP_OFFSET_SELF: usize = Self::SFX_DATA_OFFSET_SELF + Self::SFX_DATA_BYTE_SIZE;
    pub const SFX_LOOP_BYTE_SIZE: usize = 4;
    pub const SFX_BYTE_SIZE: usize = Self::SFX_LOOP_OFFSET_SELF + Self::SFX_LOOP_BYTE_SIZE;
    pub const SFX_BYTE_SIZE_TOTAL: usize =
        Self::SFX_N * (Self::SFX_LOOP_OFFSET_SELF + Self::SFX_LOOP_BYTE_SIZE);
    pub const MUSIC_PATTERNS_OFFSET: usize = Self::SFX_OFFSET + Self::SFX_BYTE_SIZE_TOTAL;
    pub const MUSIC_PATTERNS_BYTE_SIZE: usize = 11520;
    pub const MUSIC_TRACKS_OFFSET: usize =
        Self::MUSIC_PATTERNS_OFFSET + Self::MUSIC_PATTERNS_BYTE_SIZE;
    pub const MUSIC_TRACKS_BYTE_SIZE: usize = 408;
    pub const SOUND_STATE_OFFSET: usize = Self::MUSIC_TRACKS_OFFSET + Self::MUSIC_TRACKS_BYTE_SIZE;
    const SOUND_STATE_BYTE_SIZE: usize = 4;
    pub const STEREO_VOLUME_OFFSET: usize = Self::SOUND_STATE_OFFSET + Self::SOUND_STATE_BYTE_SIZE;
    const STEREO_VOLUME_BYTE_SIZE: usize = 4;
    pub const PERSISTENT_MEMORY_OFFSET: usize =
        Self::STEREO_VOLUME_OFFSET + Self::STEREO_VOLUME_BYTE_SIZE;
    pub const PERSISTENT_MEMORY_SIZE: usize = 256;
    const BYTE_PER_PMEM: usize = 4;
    pub const PERSISTENT_MEMORY_BYTE_SIZE: usize =
        Self::PERSISTENT_MEMORY_SIZE * Self::BYTE_PER_PMEM;
    pub const SPRITE_FLAGS_OFFSET: usize =
        Self::PERSISTENT_MEMORY_OFFSET + Self::PERSISTENT_MEMORY_BYTE_SIZE;
    pub const SPRITE_FLAGS_BYTE_SIZE: usize = Self::TILES_N + Self::SPRITES_N;
    pub const SYSTEM_FONT_OFFSET: usize = Self::SPRITE_FLAGS_OFFSET + Self::SPRITE_FLAGS_BYTE_SIZE;
    const SYSTEM_FONT_BYTE_SIZE: usize = 1024;
    pub const ALT_FONT_OFFSET: usize = Self::SYSTEM_FONT_OFFSET + Self::SYSTEM_FONT_BYTE_SIZE;
    const ALT_FONT_BYTE_SIZE: usize = 1024;
    pub const FONT_PARAM_OFFSET_RELATIVE: usize = Self::SYSTEM_FONT_BYTE_SIZE - 8;
    pub const GAMEPAD_MAPPING_OFFSET: usize = Self::ALT_FONT_OFFSET + Self::ALT_FONT_BYTE_SIZE;
    const GAMEPAD_MAPPING_BYTE_SIZE: usize = 32;

    pub fn new() -> Self {
        let mut ram = [0; Self::SIZE + Vram::SIZE];
        let palette_default = crate::data::tic80_palette();
        for i in 0..2 {
            for j in 0..8 {
                ram[i * Vram::SIZE + Vram::PALETTE_MAP_OFFSET + j] =
                    (j * 2 + (j * 2 + 1) * 16) as u8;
            }
            for j in 0..48 {
                ram[i * Vram::SIZE + Vram::PALETTE_OFFSET + j] = palette_default[j];
            }
            ram[i * Vram::SIZE + Vram::BLIT_SEGMENT_OFFSET] = 2;
        }
        let font_data = crate::data::tic80_font();
        for i in 0..(font_data.0.len()) {
            ram[Self::SYSTEM_FONT_OFFSET + Vram::SIZE + i] = font_data.0[i];
        }
        for i in 0..(font_data.1.len()) {
            ram[Self::ALT_FONT_OFFSET + Vram::SIZE + i] = font_data.1[i];
        }
        let key_map = crate::data::default_key_map();
        for i in 0..8 {
            ram[Self::GAMEPAD_MAPPING_OFFSET + Vram::SIZE + i] = key_map[i];
        }
        Self {
            vram: Vram::new(),
            ram,
            active_vbank: 0,
        }
    }
    pub fn set_active_vbank(&mut self, id: usize) {
        self.active_vbank = id;
    }
    pub fn get_subpixels_mut(&mut self) -> &mut PixMask {
        &mut self.vram.subpixels
    }
    pub fn clear_overlay(&mut self) {
        for i in 0..Vram::SCREEN_BYTE_SIZE {
            self.ram[i] = 0;
        }
    }
    pub unsafe fn get_unchecked(&self, index: usize) -> u8 {
        unsafe {
            *self.ram.get_unchecked(
                Vram::SIZE + index - (index < Vram::SIZE) as usize * self.active_vbank * Vram::SIZE,
            )
        }
    }
}

impl std::ops::Index<usize> for Ram {
    type Output = u8;
    fn index(&self, index: usize) -> &u8 {
        &self.ram
            [Vram::SIZE + index - (index < Vram::SIZE) as usize * self.active_vbank * Vram::SIZE]
    }
}

impl std::ops::IndexMut<usize> for Ram {
    fn index_mut(&mut self, index: usize) -> &mut u8 {
        &mut self.ram
            [Vram::SIZE + index - (index < Vram::SIZE) as usize * self.active_vbank * Vram::SIZE]
    }
}
