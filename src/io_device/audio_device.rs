#[derive(Copy, Clone)]
pub struct WheelSoundRegister {
    pub waveform: [u8; Self::WAVEFORM_LENGTH],
    pub volumn: u8,
    pub freq: u16,
}

impl WheelSoundRegister {
    const WAVEFORM_LENGTH: usize = 32;
    pub fn new() -> Self {
        Self {
            waveform: [0; Self::WAVEFORM_LENGTH],
            volumn: 0,
            freq: 0,
        }
    }
}

pub struct Speaker {
    context: web_sys::AudioContext,
    buffer: web_sys::AudioBuffer,
    phases: [f64; Self::CHANNEL_N],
    source: Option<web_sys::AudioBufferSourceNode>,
}

impl Speaker {
    const CHANNEL_N: usize = 4;
    const CLOCK_FREQ: f32 = 76800.0;
    const BUFFER_LEN: u32 = (Self::CLOCK_FREQ / 60.0) as u32;
    pub fn new(context: web_sys::AudioContext) -> Self {
        let buffer = context.create_buffer(Self::CHANNEL_N as u32, (Self::CLOCK_FREQ / 60.0) as u32, Self::CLOCK_FREQ).unwrap();
        Self {
            context,
            buffer,
            phases: [0.0; Self::CHANNEL_N],
            source: None,
        }
    }
    fn play_buffer(&mut self) {
        let source = self.context.create_buffer_source().unwrap();
        source.set_buffer(Some(&self.buffer));
        source.connect_with_audio_node(&self.context.destination()).unwrap();
        source.set_loop(true);
        if let Some(source_) = &self.source {
            source_.stop().unwrap();
        }
        source.start().unwrap();
        self.source = Some(source);
    }
}

pub trait PlayRegister {
    fn set_registers(&mut self, reg: &[WheelSoundRegister]);
}

impl PlayRegister for Speaker {
    fn set_registers(&mut self, reg: &[WheelSoundRegister]) {
        for i in 0..Self::CHANNEL_N {
            let mut arr = [0.0; Self::BUFFER_LEN as usize];
            for j in 0..Self::BUFFER_LEN as usize {
                let wave_i = (((j as f64 * Self::CLOCK_FREQ as f64 / reg[i].freq as f64).fract() + self.phases[i] as f64) * 32.0) as usize % 32;
                arr[j] = (reg[i].waveform[wave_i] * reg[i].volumn) as f32 * 2.0 / (15.0 * 15.0) - 1.0;

            }
            self.buffer.copy_to_channel(&arr, i as i32).unwrap();
            self.phases[i] = (self.phases[i] + Self::BUFFER_LEN as f64 * Self::CLOCK_FREQ as f64 / reg[i].freq as f64).fract();
        }
        self.play_buffer();
    }
}
