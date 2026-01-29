pub struct Speaker {
    context: web_sys::AudioContext,
    oscillator: web_sys::OscillatorNode,
    waveform: ([f32; 2], [f32; 2]),
    constraint: web_sys::PeriodicWaveConstraints,
}

impl Speaker {
    pub fn new(context: web_sys::AudioContext) -> Self {
        let oscillator = context.create_oscillator().unwrap();
        oscillator.frequency().set_value(440.0);
        let mut waveform = ([0.0, 0.0], [0.0, 0.0]);
        let constraint = web_sys::PeriodicWaveConstraints::new();
        constraint.set_disable_normalization(true);
        let wave = context.create_periodic_wave_with_constraints(&mut waveform.0, &mut waveform.1, &constraint).unwrap();
        oscillator.set_periodic_wave(&wave);
        oscillator.connect_with_audio_node(&context.destination()).unwrap();
        oscillator.start().unwrap();
        Self {
            context,
            oscillator,
            waveform,
            constraint
        }
    }
    pub fn start(&mut self) {
        self.waveform.0[1] = 1.0;
        let wave = self.context.create_periodic_wave_with_constraints(&mut self.waveform.0, &mut self.waveform.1, &self.constraint).unwrap();
        self.oscillator.set_periodic_wave(&wave);
    }
    pub fn stop(&mut self) {
        self.waveform.0[1] = 0.0;
        let wave = self.context.create_periodic_wave_with_constraints(&mut self.waveform.0, &mut self.waveform.1, &self.constraint).unwrap();
        self.oscillator.set_periodic_wave(&wave);
    }
}
