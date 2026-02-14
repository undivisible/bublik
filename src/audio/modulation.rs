use wasm_bindgen::prelude::*;
use web_sys::{AudioContext, AudioParam, GainNode, OscillatorNode, OscillatorType};

pub struct Lfo {
    osc: OscillatorNode,
    depth: GainNode,
    pub rate: f32,
    pub amount: f32,
}

impl Lfo {
    pub fn new(ctx: &AudioContext, rate: f32, amount: f32) -> Result<Self, JsValue> {
        let osc = ctx.create_oscillator()?;
        osc.set_type(OscillatorType::Sine);
        osc.frequency().set_value(rate);

        let depth = ctx.create_gain()?;
        depth.gain().set_value(amount);

        osc.connect_with_audio_node(&depth)?;
        osc.start()?;

        Ok(Self {
            osc,
            depth,
            rate,
            amount,
        })
    }

    pub fn connect_to(&self, param: &AudioParam) -> Result<(), JsValue> {
        self.depth.connect_with_audio_param(param)?;
        Ok(())
    }

    pub fn set_rate(&mut self, rate: f32) {
        self.rate = rate;
        self.osc.frequency().set_value(rate);
    }

    pub fn set_amount(&mut self, amount: f32) {
        self.amount = amount;
        self.depth.gain().set_value(amount);
    }

    pub fn disconnect(&self) {
        let _ = self.osc.stop();
        let _ = self.osc.disconnect();
        let _ = self.depth.disconnect();
    }
}
