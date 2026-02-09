struct Channel {
    oscillator: web_sys::OscillatorNode,
    waveform_raw: ([f32; Self::WAVEFORM_LENGTH * Self::UPSAMPLE_RATE], [f32; Self::WAVEFORM_LENGTH * Self::UPSAMPLE_RATE]),
}

impl Channel {
    const BIT_DEPTH: f32 = 15.0;
    const WAVEFORM_LENGTH: usize = 32;
    const UPSAMPLE_RATE: usize = 4;

    fn new(context: &web_sys::AudioContext, constraints: &web_sys::PeriodicWaveConstraints) -> Self {
        let mut res = Self {
            oscillator: context.create_oscillator().unwrap(),
            waveform_raw: ([0.0; Self::WAVEFORM_LENGTH * Self::UPSAMPLE_RATE], [0.0; Self::WAVEFORM_LENGTH * Self::UPSAMPLE_RATE]),
        };
        res.load_waveform(context, constraints);
        res.oscillator.connect_with_audio_node(&context.destination()).unwrap();
        res.oscillator.start().unwrap();
        res
    }
    fn set_waveform_raw(&mut self, waveform: [f32; Self::WAVEFORM_LENGTH]) {
        let waveform: [f32; Self::WAVEFORM_LENGTH * Self::UPSAMPLE_RATE] = std::array::from_fn(|i| {waveform[i / Self::UPSAMPLE_RATE]});
        const I: num::Complex::<f32> = num::Complex::<f32>::I;
        const PI: f32 = std::f32::consts::PI;
        let mut waveform_complex = [num::Complex::<f32>::ZERO; Self::WAVEFORM_LENGTH * Self::UPSAMPLE_RATE];
        // slow fourier transform
        for i in 0..Self::WAVEFORM_LENGTH * Self::UPSAMPLE_RATE {
            for j in 0..Self::WAVEFORM_LENGTH * Self::UPSAMPLE_RATE {
                waveform_complex[i] += (-I * num::Complex::<f32>::from(2.0 * PI * i as f32 * j as f32 / (Self::WAVEFORM_LENGTH * Self::UPSAMPLE_RATE) as f32)).exp() * waveform[j];
            }
        }
        
        for i in 1..Self::WAVEFORM_LENGTH {
            self.waveform_raw.0[i] = waveform_complex[i].re;
            self.waveform_raw.1[i] = waveform_complex[i].im;
        }
    }
    fn set_waveform(&mut self, waveform: [u8; Self::WAVEFORM_LENGTH], volume: u8) {
        let waveform_f = waveform.map(|i| i as f32 * volume as f32 * 2.0 / (Self::BIT_DEPTH * Self::BIT_DEPTH));
        self.set_waveform_raw(waveform_f);
    }
    fn set_freq(&self, freq: f32) {
        self.oscillator.frequency().set_value(freq);
    }
    fn load_waveform(&mut self, context: &web_sys::AudioContext, constraints: &web_sys::PeriodicWaveConstraints) {
        let wave = context.create_periodic_wave_with_constraints(&mut self.waveform_raw.0, &mut self.waveform_raw.1, constraints).unwrap();
        self.oscillator.set_periodic_wave(&wave);
    }
}

#[derive(Copy, Clone)]
pub struct WheelSoundRegister {
    pub waveform: [u8; Channel::WAVEFORM_LENGTH],
    pub volumn: u8,
    pub freq: u16,
}

impl WheelSoundRegister {
    pub fn new() -> Self {
        Self {
            waveform: [0; Channel::WAVEFORM_LENGTH],
            volumn: 0,
            freq: 0,
        }
    }
}

pub struct Speaker {
    context: web_sys::AudioContext,
    constraint: web_sys::PeriodicWaveConstraints,
    channels: [Channel; Self::CHANNEL_N]
}

impl Speaker {
    const CHANNEL_N: usize = 4;
    pub fn new(context: web_sys::AudioContext) -> Self {
        let constraint = web_sys::PeriodicWaveConstraints::new();
        constraint.set_disable_normalization(true);
        let channels = [(); Self::CHANNEL_N].map(|_| Channel::new(&context, &constraint));
        for c in &channels {
            c.set_freq(440.0);
        }
        Self {
            context,
            constraint,
            channels
        }
    }
}

pub trait PlayRegister {
    fn set_registers(&mut self, reg: &[WheelSoundRegister]);
}

impl PlayRegister for Speaker {
    fn set_registers(&mut self, reg: &[WheelSoundRegister]) {
        for i in 0..Self::CHANNEL_N {
            self.channels[i].set_waveform(reg[i].waveform, reg[i].volumn);
            self.channels[i].set_freq(32.0 * reg[i].freq as f32);
            self.channels[i].load_waveform(&self.context, &self.constraint);
        }
    }
}
