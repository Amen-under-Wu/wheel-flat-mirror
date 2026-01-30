struct Channel {
    oscillator: web_sys::OscillatorNode,
    waveform_raw: ([f32; Self::WAVEFORM_LENGTH], [f32; Self::WAVEFORM_LENGTH]),
}

impl Channel {
    const BIT_DEPTH: f32 = 15.0;
    const WAVEFORM_LENGTH: usize = 32;

    fn new(context: &web_sys::AudioContext, constraints: &web_sys::PeriodicWaveConstraints) -> Self {
        let mut res = Self {
            oscillator: context.create_oscillator().unwrap(),
            waveform_raw: ([0.0; Self::WAVEFORM_LENGTH], [0.0; Self::WAVEFORM_LENGTH]),
        };
        res.load_waveform(context, constraints);
        res.oscillator.connect_with_audio_node(&context.destination()).unwrap();
        res.oscillator.start().unwrap();
        res
    }
    fn set_waveform_raw(&mut self, waveform: [f32; Self::WAVEFORM_LENGTH]) {
        const I: num::Complex::<f32> = num::Complex::<f32>::I;
        const PI: f32 = std::f32::consts::PI;
        let mut waveform_complex = [num::Complex::<f32>::ZERO; Self::WAVEFORM_LENGTH];
        // slow fourier transform
        for i in 0..Self::WAVEFORM_LENGTH {
            for j in 0..Self::WAVEFORM_LENGTH {
                waveform_complex[i] += (-I * num::Complex::<f32>::from(2.0 * PI * i as f32 * j as f32 / Self::WAVEFORM_LENGTH as f32)).exp() * waveform[j];
            }
        }
        
        for i in 0..Self::WAVEFORM_LENGTH {
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

pub struct Speaker {
    context: web_sys::AudioContext,
    constraint: web_sys::PeriodicWaveConstraints,
    channels: [Channel; Self::CHANNEL_N]
}

impl Speaker {
    const CHANNEL_N: usize = 4;
    const SQUARE: [u8; Channel::WAVEFORM_LENGTH] = [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,15,15,15,15,15,15,15,15,15,15,15,15,15,15,15,15];
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
    pub fn start(&mut self) {
        self.channels[0].set_waveform(Self::SQUARE.clone(), 15);
        //self.channels[0].waveform_raw.0[1] = 1.0;
        self.channels[0].load_waveform(&self.context, &self.constraint);
    }
    pub fn stop(&mut self) {
        self.channels[0].set_waveform(Self::SQUARE.clone(), 0);
        //self.channels[0].waveform_raw.0[1] = 0.0;
        self.channels[0].load_waveform(&self.context, &self.constraint);
    }
}
