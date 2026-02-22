pub struct PixMask {
    data: Vec<Option<[u8; 4]>>,
}

impl PixMask {
    const W: usize = crate::cartridge::ram::Vram::SCREEN_WIDTH;
    pub fn new(size: usize) -> Self {
        PixMask {
            data: vec![None; size],
        }
    }
    pub fn set(&mut self, x: usize, y: usize, pos: usize, val: u8) {
        let idx = y * Self::W + x;
        if let Some(ref mut pix) = self.data[idx] {
            pix[pos] = val;
        } else {
            let mut pix = [0xff; 4];
            pix[pos] = val;
            self.data[idx] = Some(pix);
        }
    }
    pub fn del(&mut self, x: usize, y: usize) {
        let idx = y * Self::W + x;
        self.data[idx] = None;
    }
    pub fn get(&self, x: usize, y: usize) -> Option<[u8; 4]> {
        let idx = y * Self::W + x;
        self.data[idx]
    }
}
