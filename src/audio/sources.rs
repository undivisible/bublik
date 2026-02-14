use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{
    AudioContext, AudioProcessingEvent, GainNode, OscillatorNode, OscillatorType,
    ScriptProcessorNode,
};

use super::filters::FilterChain;

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum SourceKind {
    BrownNoise,
    PinkNoise,
    WhiteNoise,
    Theta,
    Alpha,
    Delta,
    CustomOsc(f32),
    Harmonic(f32),
    RainTexture,
}

impl SourceKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::BrownNoise => "Brown",
            Self::PinkNoise => "Pink",
            Self::WhiteNoise => "White",
            Self::Theta => "Theta",
            Self::Alpha => "Alpha",
            Self::Delta => "Delta",
            Self::CustomOsc(_) => "Osc",
            Self::Harmonic(_) => "Harmonic",
            Self::RainTexture => "Rain",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            Self::BrownNoise => "#8B4513",
            Self::PinkNoise => "#FF69B4",
            Self::WhiteNoise => "#E0E0E0",
            Self::Theta => "#00CED1",
            Self::Alpha => "#FFD700",
            Self::Delta => "#9370DB",
            Self::CustomOsc(_) => "#00FF88",
            Self::Harmonic(_) => "#FF6347",
            Self::RainTexture => "#4682B4",
        }
    }

    pub fn default_freq(&self) -> f32 {
        match self {
            Self::BrownNoise | Self::PinkNoise | Self::WhiteNoise | Self::RainTexture => 0.0,
            Self::Theta => 6.0,
            Self::Alpha => 10.0,
            Self::Delta => 2.0,
            Self::CustomOsc(f) => *f,
            Self::Harmonic(f) => *f,
        }
    }
}

pub struct SoundSource {
    pub kind: SourceKind,
    pub gain: f32,
    pub freq: f32,
    pub x: f32,
    pub y: f32,
    gain_node: GainNode,
    oscillator: Option<OscillatorNode>,
    noise_processor: Option<ScriptProcessorNode>,
    harmonic_oscs: Vec<OscillatorNode>,
    filter_chain: FilterChain,
    _closures: Vec<Closure<dyn FnMut(AudioProcessingEvent)>>,
}

impl SoundSource {
    pub fn new(
        ctx: &AudioContext,
        kind: SourceKind,
        master: &GainNode,
        x: f32,
        y: f32,
    ) -> Result<Self, JsValue> {
        let gain_node = ctx.create_gain()?;
        let gain_val = y.clamp(0.0, 1.0);
        gain_node.gain().set_value(gain_val);

        let mut filter_chain = FilterChain::new();
        let output_node = filter_chain.build(ctx, &gain_node)?;
        output_node.connect_with_audio_node(master)?;

        let mut oscillator = None;
        let mut noise_processor = None;
        let mut harmonic_oscs = Vec::new();
        let mut closures: Vec<Closure<dyn FnMut(AudioProcessingEvent)>> = Vec::new();

        let freq = match kind {
            SourceKind::Theta => 6.0,
            SourceKind::Alpha => 10.0,
            SourceKind::Delta => 2.0,
            SourceKind::CustomOsc(f) => f,
            SourceKind::Harmonic(f) => f,
            _ => x * 1000.0, // noise types use x for "color"
        };

        match kind {
            SourceKind::BrownNoise => {
                let processor = ctx.create_script_processor_with_buffer_size(4096)?;
                let last_val = std::cell::Cell::new(0.0f32);
                let closure = Closure::wrap(Box::new(move |event: AudioProcessingEvent| {
                    let output = event.output_buffer().unwrap();
                    let length = output.length() as usize;
                    let mut buf = vec![0.0f32; length];
                    let mut val = last_val.get();
                    for sample in buf.iter_mut() {
                        val += (js_sys::Math::random() as f32 * 2.0 - 1.0) * 0.02;
                        val = val.clamp(-1.0, 1.0);
                        *sample = val;
                    }
                    last_val.set(val);
                    for ch in 0..output.number_of_channels() {
                        let _ = output.copy_to_channel(&buf, ch as i32);
                    }
                }) as Box<dyn FnMut(AudioProcessingEvent)>);
                processor.set_onaudioprocess(Some(closure.as_ref().unchecked_ref()));
                processor.connect_with_audio_node(&gain_node)?;
                closures.push(closure);
                noise_processor = Some(processor);
            }
            SourceKind::PinkNoise => {
                let processor = ctx.create_script_processor_with_buffer_size(4096)?;
                let b = std::cell::RefCell::new([0.0f64; 7]);
                let closure = Closure::wrap(Box::new(move |event: AudioProcessingEvent| {
                    let output = event.output_buffer().unwrap();
                    let length = output.length() as usize;
                    let mut buf = vec![0.0f32; length];
                    let mut b = b.borrow_mut();
                    for sample in buf.iter_mut() {
                        let white = js_sys::Math::random() * 2.0 - 1.0;
                        b[0] = 0.99886 * b[0] + white * 0.0555179;
                        b[1] = 0.99332 * b[1] + white * 0.0750759;
                        b[2] = 0.96900 * b[2] + white * 0.1538520;
                        b[3] = 0.86650 * b[3] + white * 0.3104856;
                        b[4] = 0.55000 * b[4] + white * 0.5329522;
                        b[5] = -0.7616 * b[5] - white * 0.0168980;
                        let pink = b[0] + b[1] + b[2] + b[3] + b[4] + b[5] + b[6] + white * 0.5362;
                        b[6] = white * 0.115926;
                        *sample = (pink * 0.11) as f32;
                    }
                    for ch in 0..output.number_of_channels() {
                        let _ = output.copy_to_channel(&buf, ch as i32);
                    }
                }) as Box<dyn FnMut(AudioProcessingEvent)>);
                processor.set_onaudioprocess(Some(closure.as_ref().unchecked_ref()));
                processor.connect_with_audio_node(&gain_node)?;
                closures.push(closure);
                noise_processor = Some(processor);
            }
            SourceKind::WhiteNoise => {
                let processor = ctx.create_script_processor_with_buffer_size(4096)?;
                let closure = Closure::wrap(Box::new(move |event: AudioProcessingEvent| {
                    let output = event.output_buffer().unwrap();
                    let length = output.length() as usize;
                    let mut buf = vec![0.0f32; length];
                    for sample in buf.iter_mut() {
                        *sample = (js_sys::Math::random() as f32) * 2.0 - 1.0;
                    }
                    for ch in 0..output.number_of_channels() {
                        let _ = output.copy_to_channel(&buf, ch as i32);
                    }
                }) as Box<dyn FnMut(AudioProcessingEvent)>);
                processor.set_onaudioprocess(Some(closure.as_ref().unchecked_ref()));
                processor.connect_with_audio_node(&gain_node)?;
                closures.push(closure);
                noise_processor = Some(processor);
            }
            SourceKind::RainTexture => {
                let processor = ctx.create_script_processor_with_buffer_size(4096)?;
                let state = std::cell::RefCell::new([0.0f64; 4]);
                let closure = Closure::wrap(Box::new(move |event: AudioProcessingEvent| {
                    let output = event.output_buffer().unwrap();
                    let length = output.length() as usize;
                    let mut buf = vec![0.0f32; length];
                    let mut s = state.borrow_mut();
                    for sample in buf.iter_mut() {
                        let white = js_sys::Math::random() * 2.0 - 1.0;
                        // Brown noise base
                        s[0] += white * 0.02;
                        s[0] = s[0].clamp(-1.0, 1.0);
                        // Resonant peaks for rain
                        s[1] = 0.95 * s[1] + white * 0.15;
                        s[2] = 0.90 * s[2] + white * 0.08;
                        // Random droplets
                        let droplet = if js_sys::Math::random() < 0.001 {
                            (js_sys::Math::random() * 0.3) as f32
                        } else {
                            0.0
                        };
                        s[3] = s[3] * 0.98 + droplet as f64;
                        *sample = (s[0] * 0.3 + s[1] * 0.2 + s[2] * 0.15 + s[3] * 0.35) as f32;
                    }
                    for ch in 0..output.number_of_channels() {
                        let _ = output.copy_to_channel(&buf, ch as i32);
                    }
                }) as Box<dyn FnMut(AudioProcessingEvent)>);
                processor.set_onaudioprocess(Some(closure.as_ref().unchecked_ref()));
                processor.connect_with_audio_node(&gain_node)?;
                closures.push(closure);
                noise_processor = Some(processor);
            }
            SourceKind::Theta | SourceKind::Alpha | SourceKind::Delta | SourceKind::CustomOsc(_) => {
                let osc = ctx.create_oscillator()?;
                osc.set_type(OscillatorType::Sine);
                osc.frequency().set_value(freq);
                osc.connect_with_audio_node(&gain_node)?;
                osc.start()?;
                oscillator = Some(osc);
            }
            SourceKind::Harmonic(fundamental) => {
                // fundamental + overtones
                for i in 1..=6u32 {
                    let osc = ctx.create_oscillator()?;
                    osc.set_type(OscillatorType::Sine);
                    osc.frequency().set_value(fundamental * i as f32);
                    let harmonic_gain = ctx.create_gain()?;
                    harmonic_gain.gain().set_value(1.0 / i as f32);
                    osc.connect_with_audio_node(&harmonic_gain)?;
                    harmonic_gain.connect_with_audio_node(&gain_node)?;
                    osc.start()?;
                    harmonic_oscs.push(osc);
                }
            }
        }

        Ok(Self {
            kind,
            gain: gain_val,
            freq,
            x,
            y,
            gain_node,
            oscillator,
            noise_processor,
            harmonic_oscs,
            filter_chain,
            _closures: closures,
        })
    }

    pub fn set_gain(&mut self, val: f32) {
        self.gain = val.clamp(0.0, 1.0);
        self.gain_node.gain().set_value(self.gain);
    }

    pub fn set_frequency(&mut self, freq: f32) {
        self.freq = freq;
        if let Some(ref osc) = self.oscillator {
            osc.frequency().set_value(freq);
        }
        for (i, osc) in self.harmonic_oscs.iter().enumerate() {
            osc.frequency().set_value(freq * (i + 1) as f32);
        }
    }

    pub fn gain_node(&self) -> &GainNode {
        &self.gain_node
    }

    pub fn filter_chain(&self) -> &FilterChain {
        &self.filter_chain
    }

    pub fn filter_chain_mut(&mut self) -> &mut FilterChain {
        &mut self.filter_chain
    }

    pub fn disconnect(&self) {
        let _ = self.gain_node.disconnect();
        if let Some(ref osc) = self.oscillator {
            let _ = osc.stop();
            let _ = osc.disconnect();
        }
        if let Some(ref proc) = self.noise_processor {
            let _ = proc.disconnect();
        }
        for osc in &self.harmonic_oscs {
            let _ = osc.stop();
            let _ = osc.disconnect();
        }
    }
}
