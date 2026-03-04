#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkType {
    Tiles = 1,
    Sprites = 2,
    Map = 4,
    Code = 5,
    Flags = 6,
    Sfx = 9,
    Waveforms = 10,
    Palette = 12,
    Music = 14,
    Patterns = 15,
    Default = 17,
    Screen = 18,
    Binary = 19,
}

#[derive(Clone)]
pub struct Chunk {
    pub chunk_type: ChunkType,
    pub bank: u8,
    pub data: Vec<u8>,
}

#[derive(Clone)]
pub struct WheelFile {
    pub chunks: Vec<Chunk>,
}

impl WheelFile {
    pub fn new() -> Self {
        WheelFile { chunks: Vec::new() }
    }
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut chunks = Vec::new();
        let mut offset = 0;

        while offset < bytes.len() {
            let chunk_type = match bytes[offset] & 31 {
                1 => ChunkType::Tiles,
                2 => ChunkType::Sprites,
                4 => ChunkType::Map,
                5 => ChunkType::Code,
                6 => ChunkType::Flags,
                9 => ChunkType::Sfx,
                10 => ChunkType::Waveforms,
                12 => ChunkType::Palette,
                14 => ChunkType::Music,
                15 => ChunkType::Patterns,
                17 => ChunkType::Default,
                18 => ChunkType::Screen,
                19 => ChunkType::Binary,
                _ => panic!("Unknown chunk type: {}", bytes[offset]),
            };

            let bank = bytes[offset] >> 5;
            let length = u16::from_le_bytes([bytes[offset + 1], bytes[offset + 2]]) as usize;
            let data = bytes[offset + 4..offset + 4 + length].to_vec();

            chunks.push(Chunk {
                chunk_type,
                bank,
                data,
            });

            offset += 4 + length;
        }

        WheelFile { chunks }
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        for chunk in &self.chunks {
            let chunk_type_byte = (chunk.chunk_type as u8) | (chunk.bank << 5);
            let length_bytes = (chunk.data.len() as u16).to_le_bytes();

            bytes.push(chunk_type_byte);
            bytes.extend_from_slice(&length_bytes);
            bytes.push(0); // Reserved byte
            bytes.extend_from_slice(&chunk.data);
        }

        bytes
    }

    pub fn get_chunk(&self, chunk_type: ChunkType, bank: u8) -> Option<&Chunk> {
        self.chunks.iter().find(|chunk| {
            chunk.chunk_type == chunk_type && chunk.bank == bank
        })
    }
    pub fn set_chunk(&mut self, chunk_type: ChunkType, bank: u8, data: Vec<u8>) {
        if let Some(chunk) = self.chunks.iter_mut().find(|chunk| {
            chunk.chunk_type == chunk_type && chunk.bank == bank
        }) {
            chunk.data = data;
        } else {
            self.chunks.push(Chunk {
                chunk_type,
                bank,
                data,
            });
        }
    }
}

pub trait Savable {
    fn save(&self) -> WheelFile;
    fn load(data: WheelFile) -> Self;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_wheel_file() {
        let f = WheelFile::from_bytes(include_bytes!("fallspire.tic"));
        for chunk in &f.chunks {
            println!("Chunk type: {:?}, bank: {}, data length: {}", chunk.chunk_type, chunk.bank, chunk.data.len());
        }
    }
}
