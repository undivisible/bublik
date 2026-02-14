use wasm_bindgen::prelude::*;
use web_sys::{AudioContext, GainNode};

use super::sources::SoundSource;

pub struct AudioEngine {
    ctx: AudioContext,
    master_gain: GainNode,
    sources: Vec<SoundSource>,
    playing: bool,
}

impl AudioEngine {
    pub fn new() -> Result<Self, JsValue> {
        let ctx = AudioContext::new()?;
        let master_gain = ctx.create_gain()?;
        master_gain.gain().set_value(0.7);
        master_gain.connect_with_audio_node(&ctx.destination())?;

        Ok(Self {
            ctx,
            master_gain,
            sources: Vec::new(),
            playing: false,
        })
    }

    pub fn context(&self) -> &AudioContext {
        &self.ctx
    }

    pub fn master_gain_node(&self) -> &GainNode {
        &self.master_gain
    }

    pub fn set_master_volume(&self, vol: f32) {
        self.master_gain.gain().set_value(vol.clamp(0.0, 1.0));
    }

    pub fn master_volume(&self) -> f32 {
        self.master_gain.gain().value()
    }

    pub fn add_source(&mut self, source: SoundSource) {
        self.sources.push(source);
    }

    pub fn remove_source(&mut self, index: usize) {
        if index < self.sources.len() {
            self.sources[index].disconnect();
            self.sources.remove(index);
        }
    }

    pub fn sources(&self) -> &[SoundSource] {
        &self.sources
    }

    pub fn sources_mut(&mut self) -> &mut Vec<SoundSource> {
        &mut self.sources
    }

    pub fn resume(&mut self) -> Result<(), JsValue> {
        let _ = self.ctx.resume()?;
        self.playing = true;
        Ok(())
    }

    pub fn suspend(&mut self) -> Result<(), JsValue> {
        let _ = self.ctx.suspend()?;
        self.playing = false;
        Ok(())
    }

    pub fn toggle(&mut self) -> Result<bool, JsValue> {
        if self.playing {
            self.suspend()?;
        } else {
            self.resume()?;
        }
        Ok(self.playing)
    }

    pub fn is_playing(&self) -> bool {
        self.playing
    }

    pub fn current_time(&self) -> f64 {
        self.ctx.current_time()
    }
}
