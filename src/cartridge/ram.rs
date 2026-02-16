#[derive(Clone, Copy)]
struct Vram {
    vbanks: [[u8; Self::SIZE]; Self::VBANK_N],
    active_vbank: usize,
}

impl Vram {
    const SIZE: usize = 16384;
    const VBANK_N: usize = 2;
    const SCREEN_WIDTH: usize = 240;
    const SCREEN_HEIGHT: usize = 136;
    const BPP: usize = 4; // bits per pixel
    const SCREEN_BYTE_SIZE: usize = Self::SCREEN_WIDTH * Self::SCREEN_HEIGHT * Self::BPP / 8;
    const PALETTE_OFFSET: usize = Self::SCREEN_BYTE_SIZE;
    const PALETTE_SIZE: usize = 16;
    const PALETTE_BYTE_SIZE: usize = Self::PALETTE_SIZE * 3;
    const PALETTE_MAP_OFFSET: usize = Self::PALETTE_OFFSET + Self::PALETTE_BYTE_SIZE;
    const PALETTE_MAP_BYTE_SIZE: usize = Self::PALETTE_SIZE * Self::PALETTE_SIZE.ilog2() as usize / 8;
    const BORDER_COLOR_OFFSET: usize = Self::PALETTE_MAP_OFFSET + Self::PALETTE_MAP_BYTE_SIZE;
    const BORDER_COLOR_BYTE_SIZE: usize = 1;
    const SCREEN_OFFSET_OFFSET: usize = Self::BORDER_COLOR_OFFSET + Self::BORDER_COLOR_BYTE_SIZE;
    const SCREEN_OFFSET_BYTE_SIZE: usize = 2;
    const MOUSE_CURSOR_OFFSET: usize = Self::SCREEN_OFFSET_OFFSET + Self::SCREEN_OFFSET_BYTE_SIZE;
    const MOUSE_CURSOR_BYTE_SIZE: usize = 1;
    const BLIT_SEGMENT_OFFSET: usize = Self::MOUSE_CURSOR_OFFSET + Self::MOUSE_CURSOR_BYTE_SIZE;

    fn new() -> Self {
        Self {
            vbanks: [[0; Self::SIZE]; Self::VBANK_N],
            active_vbank: 0,
        }
    }
    fn set_active_bank(&mut self, bank: usize) {
        self.active_vbank = bank;
    }
}

impl std::ops::Index<usize> for Vram {
    type Output = u8;
    fn index(&self, index: usize) -> &u8 {
        &self.vbanks[self.active_vbank][index]
    }
}

impl std::ops::IndexMut<usize> for Vram {
    fn index_mut(&mut self, index: usize) -> &mut u8 {
        &mut self.vbanks[self.active_vbank][index]
    }
}


#[derive(Clone, Copy)]
struct Ram {
    vram: Vram,
    ram: [u8; Self::SIZE - Vram::SIZE],
}

impl Ram {
    const SIZE: usize = 0x18000;
    const TILES_OFFSET: usize = Vram::SIZE;
    const BPP: usize = 4;
    const TILES_N: usize = 256;
    const TILE_W: usize = 8;
    const TILE_H: usize = 8;
    const TILES_BYTE_SIZE: usize = Self::TILE_W * Self::TILE_H * Self::TILES_N * Self::BPP / 8;
    const SPRITES_OFFSET: usize = Self::TILES_OFFSET + Self::TILES_BYTE_SIZE;
    const SPRITES_N: usize = 256;
    const SPRITE_W: usize = 8;
    const SPRITE_H: usize = 8;
    const SPRITES_BYTE_SIZE: usize = Self::SPRITE_W * Self::SPRITE_H * Self::SPRITES_N * Self::BPP / 8;
    const MAP_OFFSET: usize = Self::SPRITES_OFFSET + Self::SPRITES_BYTE_SIZE;
    const MAP_W: usize = 240;
    const MAP_H: usize = 136;
    const MAP_TILE_BYTE_SIZE: usize = 1;
    const MAP_BYTE_SIZE: usize = Self::MAP_W * Self::MAP_H * Self::MAP_TILE_BYTE_SIZE;
    const GAMEPADS_OFFSET: usize = Self::MAP_OFFSET + Self::MAP_BYTE_SIZE;
    const GAMEPADS_BYTE_SIZE: usize = 4;
    const MOUSE_OFFSET: usize = Self::GAMEPADS_OFFSET + Self::GAMEPADS_BYTE_SIZE;
    const MOUSE_BYTE_SIZE: usize = 4;
    const KEYBOARD_OFFSET: usize = Self::MOUSE_OFFSET + Self::MOUSE_BYTE_SIZE;
    const KEYBOARD_BYTE_SIZE: usize = 4;
    const SFX_STATE_OFFSET: usize = Self::KEYBOARD_OFFSET + Self::KEYBOARD_BYTE_SIZE;
    const SFX_STATE_BYTE_SIZE: usize = 16;
    const SOUND_REGISTERS_OFFSET: usize = Self::SFX_STATE_OFFSET + Self::SFX_STATE_BYTE_SIZE;
    const SOUND_REGISTERS_BYTE_SIZE: usize = 72;
    const WAVEFORMS_OFFSET: usize = Self::SOUND_REGISTERS_OFFSET + Self::SOUND_REGISTERS_BYTE_SIZE;
    const WAVEFORMS_N: usize = 16;
    const WAVEFORM_SAMPLE_N: usize = 32;
    const WAVEFORM_BPS: usize = 4; // bits per sample
    const WAVEFORMS_BYTE_SIZE: usize = Self::WAVEFORMS_N * Self::WAVEFORM_SAMPLE_N * Self::WAVEFORM_BPS / 8;
    const SFX_OFFSET: usize = Self::WAVEFORMS_OFFSET + Self::WAVEFORMS_BYTE_SIZE;
    const SFX_BYTE_SIZE: usize = 4224;
    const MUSIC_PATTERNS_OFFSET: usize = Self::SFX_OFFSET + Self::SFX_BYTE_SIZE;
    const MUSIC_PATTERNS_BYTE_SIZE: usize = 11520;
    const MUSIC_TRACKS_OFFSET: usize = Self::MUSIC_PATTERNS_OFFSET + Self::MUSIC_PATTERNS_BYTE_SIZE;
    const MUSIC_TRACKS_BYTE_SIZE: usize = 408;
    const SOUND_STATE_OFFSET: usize = Self::MUSIC_TRACKS_OFFSET + Self::MUSIC_TRACKS_BYTE_SIZE;
    const SOUND_STATE_BYTE_SIZE: usize = 4;
    const STEREO_VOLUME_OFFSET: usize = Self::SOUND_STATE_OFFSET + Self::SOUND_STATE_BYTE_SIZE;
    const STEREO_VOLUME_BYTE_SIZE: usize = 4;
    const PERSISTENT_MEMORY_OFFSET: usize = Self::STEREO_VOLUME_OFFSET + Self::STEREO_VOLUME_BYTE_SIZE;
    const PERSISTENT_MEMORY_BYTE_SIZE: usize = 1024;
    const SPRITE_FLAGS_OFFSET: usize = Self::PERSISTENT_MEMORY_OFFSET + Self::PERSISTENT_MEMORY_BYTE_SIZE;
    const SPRITE_FLAGS_BYTE_SIZE: usize = Self::TILES_N + Self::SPRITES_N;
    const SYSTEM_FONT_OFFSET: usize = Self::SPRITE_FLAGS_OFFSET + Self::SPRITE_FLAGS_BYTE_SIZE;
    const SYSTEM_FONT_BYTE_SIZE: usize = 2048;
    const GAMEPAD_MAPPING_OFFSET: usize = Self::SYSTEM_FONT_OFFSET + Self::SYSTEM_FONT_BYTE_SIZE;
    const GAMEPAD_MAPPING_BYTE_SIZE: usize = 32;
}

impl std::ops::Index<usize> for Ram {
    type Output = u8;
    fn index(&self, index: usize) -> &u8 {
        if index < Vram::SIZE {
            &self.vram[index]
        } else {
            &self.ram[index - Vram::SIZE]
        }
    }
}

impl std::ops::IndexMut<usize> for Ram {
    fn index_mut(&mut self, index: usize) -> &mut u8 {
        if index < Vram::SIZE {
            &mut self.vram[index]
        } else {
            &mut self.ram[index - Vram::SIZE]
        }
    }
}
