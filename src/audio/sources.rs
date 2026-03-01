use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{
    AudioContext, AudioProcessingEvent, GainNode, OscillatorNode, OscillatorType,
    ScriptProcessorNode,
};

use super::filters::FilterChain;
use crate::audio::binaural::BinauralBeat;

fn log_err(context: &str, err: &JsValue) {
    web_sys::console::error_2(
        &JsValue::from_str(&format!("[bublik audio] {}", context)),
        err,
    );
}

type AudioProcessClosure = Closure<dyn FnMut(AudioProcessingEvent)>;

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

    pub fn is_noise(&self) -> bool {
        matches!(
            self,
            Self::BrownNoise | Self::PinkNoise | Self::WhiteNoise | Self::RainTexture
        )
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

/// Build AM-synthesis brainwave source (carrier modulated by brainwave freq).
/// Returns (carrier, modulator, [mod_gain, am_gain]).
fn build_brainwave(
    ctx: &AudioContext,
    gain_node: &GainNode,
    freq: f32,
) -> Result<(OscillatorNode, OscillatorNode, Vec<GainNode>), JsValue> {
    let carrier = ctx.create_oscillator()?;
    carrier.set_type(OscillatorType::Sine);
    carrier.frequency().set_value(150.0);

    let modulator = ctx.create_oscillator()?;
    modulator.set_type(OscillatorType::Sine);
    modulator.frequency().set_value(freq);

    let mod_gain = ctx.create_gain()?;
    mod_gain.gain().set_value(0.5);

    let am_gain = ctx.create_gain()?;
    am_gain.gain().set_value(0.5);

    carrier
        .connect_with_audio_node(&am_gain)
        .inspect_err(|e| {
            log_err("carrier->am_gain", e);
        })?;
    am_gain
        .connect_with_audio_node(gain_node)
        .inspect_err(|e| {
            log_err("am_gain->gain_node", e);
        })?;
    modulator
        .connect_with_audio_node(&mod_gain)
        .inspect_err(|e| {
            log_err("modulator->mod_gain", e);
        })?;
    mod_gain
        .connect_with_audio_param(&am_gain.gain())
        .inspect_err(|e| {
            log_err("mod_gain->am_gain.gain", e);
        })?;

    carrier.start()?;
    modulator.start()?;

    Ok((carrier, modulator, vec![mod_gain, am_gain]))
}

/// Build harmonic series (fundamental + overtones).
/// Returns (oscillators, gain_nodes).
fn build_harmonics(
    ctx: &AudioContext,
    gain_node: &GainNode,
    fundamental: f32,
) -> Result<(Vec<OscillatorNode>, Vec<GainNode>), JsValue> {
    let mut oscs = Vec::new();
    let mut gains = Vec::new();

    for i in 1..=6u32 {
        let osc = ctx.create_oscillator()?;
        osc.set_type(OscillatorType::Sine);
        osc.frequency().set_value(fundamental * i as f32);
        let harmonic_gain = ctx.create_gain()?;
        harmonic_gain.gain().set_value(1.0 / i as f32);
        osc.connect_with_audio_node(&harmonic_gain)
            .inspect_err(|e| {
                log_err(&format!("harmonic[{}]->gain", i), e);
            })?;
        harmonic_gain
            .connect_with_audio_node(gain_node)
            .inspect_err(|e| {
                log_err(&format!("harmonic_gain[{}]->gain_node", i), e);
            })?;
        osc.start()?;
        oscs.push(osc);
        gains.push(harmonic_gain);
    }

    Ok((oscs, gains))
}

/// Build a noise ScriptProcessorNode with the given sample callback.
fn build_noise<F>(
    ctx: &AudioContext,
    gain_node: &GainNode,
    fill_fn: F,
) -> Result<(ScriptProcessorNode, AudioProcessClosure), JsValue>
where
    F: FnMut(&mut [f32]) + 'static,
{
    let processor = ctx.create_script_processor_with_buffer_size(4096)?;
    let fill = std::cell::RefCell::new(fill_fn);
    let scratch = std::cell::RefCell::new(Vec::<f32>::new());
    let closure = Closure::wrap(Box::new(move |event: AudioProcessingEvent| {
        let Ok(output) = event.output_buffer() else { return };
        let length = output.length() as usize;
        let mut buf = scratch.borrow_mut();
        if buf.len() != length {
            buf.resize(length, 0.0);
        }
        (fill.borrow_mut())(&mut buf[..]);
        for ch in 0..output.number_of_channels() {
            let _ = output.copy_to_channel(&buf, ch as i32);
        }
    }) as Box<dyn FnMut(AudioProcessingEvent)>);
    processor.set_onaudioprocess(Some(closure.as_ref().unchecked_ref()));
    processor
        .connect_with_audio_node(gain_node)
        .inspect_err(|e| {
            log_err("noise->gain_node", e);
        })?;
    Ok((processor, closure))
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
    extra_gains: Vec<GainNode>,
    binaural_beat: Option<BinauralBeat>,
    filter_chain: FilterChain,
    _closures: Vec<AudioProcessClosure>,
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
        output_node
            .connect_with_audio_node(master)
            .inspect_err(|e| {
                log_err("filter->master", e);
            })?;

        let mut oscillator = None;
        let mut noise_processor = None;
        let mut harmonic_oscs = Vec::new();
        let mut extra_gains: Vec<GainNode> = Vec::new();
        let mut closures: Vec<AudioProcessClosure> = Vec::new();

        let freq = match kind {
            SourceKind::Theta => 6.0,
            SourceKind::Alpha => 10.0,
            SourceKind::Delta => 2.0,
            SourceKind::CustomOsc(f) => f,
            SourceKind::Harmonic(f) => f,
            _ => x * 1000.0,
        };

        match kind {
            SourceKind::BrownNoise => {
                let last_val = std::cell::Cell::new(0.0f32);
                let (proc, closure) = build_noise(ctx, &gain_node, move |buf| {
                    let mut val = last_val.get();
                    for sample in buf.iter_mut() {
                        val += (js_sys::Math::random() as f32 * 2.0 - 1.0) * 0.02;
                        val = val.clamp(-1.0, 1.0);
                        *sample = val;
                    }
                    last_val.set(val);
                })?;
                closures.push(closure);
                noise_processor = Some(proc);
            }
            SourceKind::PinkNoise => {
                let b = std::cell::RefCell::new([0.0f64; 7]);
                let (proc, closure) = build_noise(ctx, &gain_node, move |buf| {
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
                })?;
                closures.push(closure);
                noise_processor = Some(proc);
            }
            SourceKind::WhiteNoise => {
                let (proc, closure) = build_noise(ctx, &gain_node, move |buf| {
                    for sample in buf.iter_mut() {
                        *sample = (js_sys::Math::random() as f32 * 2.0 - 1.0) * 0.1;
                    }
                })?;
                closures.push(closure);
                noise_processor = Some(proc);
            }
            SourceKind::RainTexture => {
                // Rain synthesis: dense droplet impacts + gentle patter + ambient wash
                // 8 state slots: [0-3] droplet envelopes at different rates,
                // [4] pink-ish wash, [5] splatter accumulator, [6-7] pink filter state
                let state = std::cell::RefCell::new([0.0f64; 8]);
                let (proc, closure) = build_noise(ctx, &gain_node, move |buf| {
                    let mut s = state.borrow_mut();
                    for sample in buf.iter_mut() {
                        let white = js_sys::Math::random() * 2.0 - 1.0;

                        // Light ambient wash (less low-freq, more high-freq hiss)
                        s[6] = 0.990 * s[6] + white * 0.015;
                        s[7] = 0.975 * s[7] + white * 0.03;
                        s[4] = s[6] + s[7];

                        // Dense small droplets (~20% chance — more prominent)
                        if js_sys::Math::random() < 0.20 {
                            s[0] += js_sys::Math::random() * 0.10;
                        }
                        s[0] *= 0.65; // Faster decay — crisper transients

                        // Medium droplets (~3% chance)
                        if js_sys::Math::random() < 0.03 {
                            s[1] += js_sys::Math::random() * 0.18;
                        }
                        s[1] *= 0.82; // Medium decay

                        // Large occasional drops (~0.4% chance)
                        if js_sys::Math::random() < 0.004 {
                            s[2] += js_sys::Math::random() * 0.28;
                        }
                        s[2] *= 0.90; // Slower decay

                        // Splatter bursts (~0.15% chance)
                        if js_sys::Math::random() < 0.0015 {
                            s[3] += 0.3 + js_sys::Math::random() * 0.2;
                        }
                        s[3] *= 0.94;
                        let splatter_crackle = if s[3] > 0.01 {
                            (js_sys::Math::random() * s[3] * 0.5) as f32
                        } else {
                            0.0
                        };

                        *sample = (s[4] * 0.06         // lighter ambient wash
                                 + s[0] * 0.42         // more prominent small drops
                                 + s[1] * 0.28         // medium drops
                                 + s[2] * 0.16         // large drops
                                 + splatter_crackle as f64 * 0.08) as f32;
                    }
                })?;
                closures.push(closure);
                noise_processor = Some(proc);
            }
            SourceKind::Theta | SourceKind::Alpha | SourceKind::Delta => {
                let (carrier, modulator, gains) = build_brainwave(ctx, &gain_node, freq)?;
                oscillator = Some(carrier);
                harmonic_oscs.push(modulator);
                extra_gains.extend(gains);
            }
            SourceKind::CustomOsc(_) => {
                let osc = ctx.create_oscillator()?;
                osc.set_type(OscillatorType::Sine);
                osc.frequency().set_value(freq);
                osc.connect_with_audio_node(&gain_node)
                    .inspect_err(|e| {
                        log_err("osc->gain_node", e);
                    })?;
                osc.start()?;
                oscillator = Some(osc);
            }
            SourceKind::Harmonic(fundamental) => {
                let (oscs, gains) = build_harmonics(ctx, &gain_node, fundamental)?;
                harmonic_oscs = oscs;
                extra_gains = gains;
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
            extra_gains,
            binaural_beat: None,
            filter_chain,
            _closures: closures,
        })
    }

    pub fn set_gain(&mut self, val: f32) {
        self.gain = val.clamp(0.0, 1.0);
        self.gain_node.gain().set_value(self.gain);
        if let Some(ref mut b) = self.binaural_beat {
            b.set_gain(self.gain);
        }
    }

    pub fn set_frequency(&mut self, freq: f32) {
        self.freq = freq;

        if let Some(ref mut b) = self.binaural_beat {
            b.set_base_freq(freq);
            return;
        }

        if let Some(ref osc) = self.oscillator {
            osc.frequency().set_value(freq);
        }
        for (i, osc) in self.harmonic_oscs.iter().enumerate() {
            if matches!(
                self.kind,
                SourceKind::Theta | SourceKind::Alpha | SourceKind::Delta
            ) {
                // Don't change modulator freq (stored in harmonic_oscs[0])
            } else {
                osc.frequency().set_value(freq * (i + 1) as f32);
            }
        }
    }

    /// Toggle binaural for this source. Routes through gain_node so filters apply.
    pub fn toggle_binaural(
        &mut self,
        ctx: &AudioContext,
        _master: &GainNode,
        beat_freq: f32,
    ) -> Result<bool, JsValue> {
        if self.binaural_beat.is_some() {
            if let Some(b) = self.binaural_beat.take() {
                b.disconnect();
            }
            // Reconnect normal generators
            self.reconnect_generators();
            self.gain_node.gain().set_value(self.gain);
            Ok(false)
        } else {
            // For brainwave sources (Theta, Alpha, Delta), use carrier frequency (150Hz)
            // instead of the modulation frequency (6-10Hz) which would be inaudible
            let base_freq = match self.kind {
                SourceKind::Theta | SourceKind::Alpha | SourceKind::Delta => 150.0,
                _ => self.freq,
            };
            
            // Route binaural through gain_node so filters apply
            let mut b = BinauralBeat::new(ctx, &self.gain_node, base_freq, beat_freq)?;
            b.set_gain(1.0); // gain_node already controls volume

            // Mute normal source generators but keep gain_node active for binaural
            // We disconnect generators by setting their contributions to zero
            if let Some(ref osc) = self.oscillator {
                let _ = osc.disconnect();
            }
            if let Some(ref proc) = self.noise_processor {
                let _ = proc.disconnect();
            }
            for osc in &self.harmonic_oscs {
                let _ = osc.disconnect();
            }

            self.binaural_beat = Some(b);
            Ok(true)
        }
    }

    /// Reconnect normal source generators after binaural is disabled.
    fn reconnect_generators(&self) {
        if let Some(ref osc) = self.oscillator {
            let _ = osc.connect_with_audio_node(&self.gain_node);
        }
        if let Some(ref proc) = self.noise_processor {
            let _ = proc.connect_with_audio_node(&self.gain_node);
        }
        // For brainwave sources, the AM chain connects through extra_gains
        // For harmonics, each osc connects through its own extra gain
        match self.kind {
            SourceKind::Theta | SourceKind::Alpha | SourceKind::Delta => {
                // Carrier -> am_gain (extra_gains[1]) -> gain_node is the chain
                // Just reconnect carrier to am_gain
                if let (Some(carrier), Some(am_gain)) = (self.oscillator.as_ref(), self.extra_gains.get(1)) {
                    let _ = carrier.connect_with_audio_node(am_gain);
                    let _ = am_gain.connect_with_audio_node(&self.gain_node);
                }
                // Modulator -> mod_gain -> am_gain.gain
                if let (Some(modulator), Some(mod_gain), Some(am_gain)) = (
                    self.harmonic_oscs.first(),
                    self.extra_gains.first(),
                    self.extra_gains.get(1),
                ) {
                    let _ = modulator.connect_with_audio_node(mod_gain);
                    let _ = mod_gain.connect_with_audio_param(&am_gain.gain());
                }
            }
            SourceKind::Harmonic(_) => {
                for (osc, gain) in self.harmonic_oscs.iter().zip(self.extra_gains.iter()) {
                    let _ = osc.connect_with_audio_node(gain);
                    let _ = gain.connect_with_audio_node(&self.gain_node);
                }
            }
            _ => {
                // Simple sources: osc/noise -> gain_node (already handled above)
            }
        }
    }

    pub fn is_binaural_active(&self) -> bool {
        self.binaural_beat.is_some()
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
    
    pub fn rebuild_filters(&mut self, ctx: &AudioContext, master: &GainNode) -> Result<(), JsValue> {
        self.filter_chain.rebuild(ctx, &self.gain_node, &master.clone().into())
    }

    pub fn set_filter_cutoff(&mut self, freq: f32) {
        self.filter_chain.set_cutoff(freq);
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
        for g in &self.extra_gains {
            let _ = g.disconnect();
        }
        if let Some(ref b) = self.binaural_beat {
            b.disconnect();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_kind_is_noise() {
        assert!(SourceKind::BrownNoise.is_noise());
        assert!(SourceKind::PinkNoise.is_noise());
        assert!(SourceKind::WhiteNoise.is_noise());
        assert!(SourceKind::RainTexture.is_noise());
        assert!(!SourceKind::Theta.is_noise());
        assert!(!SourceKind::Alpha.is_noise());
        assert!(!SourceKind::Delta.is_noise());
        assert!(!SourceKind::CustomOsc(440.0).is_noise());
        assert!(!SourceKind::Harmonic(220.0).is_noise());
    }

    #[test]
    fn test_source_kind_default_freq() {
        assert_eq!(SourceKind::BrownNoise.default_freq(), 0.0);
        assert_eq!(SourceKind::Theta.default_freq(), 6.0);
        assert_eq!(SourceKind::Alpha.default_freq(), 10.0);
        assert_eq!(SourceKind::Delta.default_freq(), 2.0);
        assert_eq!(SourceKind::CustomOsc(432.0).default_freq(), 432.0);
        assert_eq!(SourceKind::Harmonic(220.0).default_freq(), 220.0);
    }

    #[test]
    fn test_source_kind_color_valid_hex() {
        let kinds = [
            SourceKind::BrownNoise,
            SourceKind::PinkNoise,
            SourceKind::WhiteNoise,
            SourceKind::Theta,
            SourceKind::Alpha,
            SourceKind::Delta,
            SourceKind::CustomOsc(440.0),
            SourceKind::Harmonic(220.0),
            SourceKind::RainTexture,
        ];
        for kind in &kinds {
            let color = kind.color();
            assert!(color.starts_with('#'), "Color should start with #: {}", color);
            assert_eq!(color.len(), 7, "Color should be 7 chars: {}", color);
        }
    }

    #[test]
    fn test_source_kind_serialization() {
        let kind = SourceKind::CustomOsc(432.0);
        let json = serde_json::to_string(&kind).unwrap();
        let deserialized: SourceKind = serde_json::from_str(&json).unwrap();
        assert_eq!(kind, deserialized);
    }
}
