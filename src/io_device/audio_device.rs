pub struct Speaker {
    context: web_sys::AudioContext,
    oscillator: web_sys::OscillatorNode,
    waveform: ([f32; 2], [f32; 2])
}

impl Speaker {
    pub fn new(context: web_sys::AudioContext) -> Self {
        let oscillator = context.create_oscillator().unwrap();
        oscillator.frequency().set_value(440.0);
        let mut waveform = ([0.0, 0.0], [0.0, 0.0]);
        let wave = context.create_periodic_wave(&mut waveform.0, &mut waveform.1).unwrap();
        oscillator.set_periodic_wave(&wave);
        oscillator.connect_with_audio_node(&context.destination()).unwrap();
        oscillator.start().unwrap();
        Self {
            context,
            oscillator,
            waveform
        }
    }
    pub fn start(&mut self) {
        self.waveform.0[1] = 1.0;
        let wave = self.context.create_periodic_wave(&mut self.waveform.0, &mut self.waveform.1).unwrap();
        self.oscillator.set_periodic_wave(&wave);
    }
    pub fn stop(&mut self) {
        self.waveform.0[1] = 0.0;
        let wave = self.context.create_periodic_wave(&mut self.waveform.0, &mut self.waveform.1).unwrap();
        self.oscillator.set_periodic_wave(&wave);
    }
}
