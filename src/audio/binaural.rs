use wasm_bindgen::prelude::*;
use web_sys::{
    AudioContext, ChannelMergerNode, GainNode, OscillatorNode, OscillatorType,
};

pub struct BinauralBeat {
    left_osc: OscillatorNode,
    right_osc: OscillatorNode,
    left_gain: GainNode,
    right_gain: GainNode,
    gain_node: GainNode,
    merger: ChannelMergerNode,
    pub base_freq: f32,
    pub beat_freq: f32,
    pub gain: f32,
    active: bool,
}

impl BinauralBeat {
    pub fn new(
        ctx: &AudioContext,
        master: &GainNode,
        base_freq: f32,
        beat_freq: f32,
    ) -> Result<Self, JsValue> {
        let left_osc = ctx.create_oscillator()?;
        let right_osc = ctx.create_oscillator()?;
        left_osc.set_type(OscillatorType::Sine);
        right_osc.set_type(OscillatorType::Sine);

        left_osc.frequency().set_value(base_freq);
        right_osc.frequency().set_value(base_freq + beat_freq);

        let gain_node = ctx.create_gain()?;
        gain_node.gain().set_value(0.5);

        let merger = ctx.create_channel_merger_with_number_of_inputs(2)?;

        let left_gain = ctx.create_gain()?;
        left_gain.gain().set_value(1.0);
        let right_gain = ctx.create_gain()?;
        right_gain.gain().set_value(1.0);

        left_osc.connect_with_audio_node(&left_gain)?;
        right_osc.connect_with_audio_node(&right_gain)?;

        left_gain.connect_with_audio_node_and_output_and_input(&merger, 0, 0)?;
        right_gain.connect_with_audio_node_and_output_and_input(&merger, 0, 1)?;

        merger.connect_with_audio_node(&gain_node)?;
        gain_node.connect_with_audio_node(master)?;

        left_osc.start()?;
        right_osc.start()?;

        Ok(Self {
            left_osc,
            right_osc,
            left_gain,
            right_gain,
            gain_node,
            merger,
            base_freq,
            beat_freq,
            gain: 0.5,
            active: true,
        })
    }

    pub fn set_base_freq(&mut self, freq: f32) {
        self.base_freq = freq;
        self.left_osc.frequency().set_value(freq);
        self.right_osc.frequency().set_value(freq + self.beat_freq);
    }

    pub fn set_beat_freq(&mut self, freq: f32) {
        self.beat_freq = freq;
        self.right_osc.frequency().set_value(self.base_freq + freq);
    }

    pub fn set_gain(&mut self, val: f32) {
        self.gain = val.clamp(0.0, 1.0);
        self.gain_node.gain().set_value(self.gain);
    }

    pub fn beat_zone(freq: f32) -> &'static str {
        match freq {
            f if f < 4.0 => "Delta",
            f if f < 8.0 => "Theta",
            f if f < 13.0 => "Alpha",
            f if f < 30.0 => "Beta",
            _ => "Gamma",
        }
    }

    pub fn disconnect(&self) {
        let _ = self.left_osc.stop();
        let _ = self.right_osc.stop();
        let _ = self.left_osc.disconnect();
        let _ = self.right_osc.disconnect();
        let _ = self.left_gain.disconnect();
        let _ = self.right_gain.disconnect();
        let _ = self.merger.disconnect();
        let _ = self.gain_node.disconnect();
    }
}
