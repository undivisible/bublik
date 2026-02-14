use wasm_bindgen::prelude::*;
use web_sys::{AudioContext, AudioNode, BiquadFilterNode, BiquadFilterType, GainNode};

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum FilterKind {
    Lowpass,
    Highpass,
    Bandpass,
    Notch,
}

impl FilterKind {
    pub fn to_biquad_type(self) -> BiquadFilterType {
        match self {
            Self::Lowpass => BiquadFilterType::Lowpass,
            Self::Highpass => BiquadFilterType::Highpass,
            Self::Bandpass => BiquadFilterType::Bandpass,
            Self::Notch => BiquadFilterType::Notch,
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct FilterSpec {
    pub kind: FilterKind,
    pub frequency: f32,
    pub q: f32,
    pub enabled: bool,
}

impl Default for FilterSpec {
    fn default() -> Self {
        Self {
            kind: FilterKind::Lowpass,
            frequency: 1000.0,
            q: 1.0,
            enabled: false,
        }
    }
}

pub struct FilterChain {
    pub specs: Vec<FilterSpec>,
    nodes: Vec<BiquadFilterNode>,
}

impl FilterChain {
    pub fn new() -> Self {
        Self {
            specs: vec![FilterSpec::default()],
            nodes: Vec::new(),
        }
    }

    pub fn with_lowpass(freq: f32, q: f32) -> Self {
        Self {
            specs: vec![FilterSpec {
                kind: FilterKind::Lowpass,
                frequency: freq,
                q,
                enabled: true,
            }],
            nodes: Vec::new(),
        }
    }

    /// Builds the filter chain, connecting input -> filters.
    /// Returns the final output AudioNode that should be connected to master.
    pub fn build(
        &mut self,
        ctx: &AudioContext,
        input: &GainNode,
    ) -> Result<AudioNode, JsValue> {
        self.nodes.clear();

        let active_specs: Vec<&FilterSpec> = self.specs.iter().filter(|s| s.enabled).collect();

        if active_specs.is_empty() {
            let node: AudioNode = input.clone().into();
            return Ok(node);
        }

        let mut last_node: AudioNode = input.clone().into();

        for spec in &active_specs {
            let filter = ctx.create_biquad_filter()?;
            filter.set_type(spec.kind.to_biquad_type());
            filter.frequency().set_value(spec.frequency);
            filter.q().set_value(spec.q);
            last_node.connect_with_audio_node(&filter)?;
            last_node = filter.clone().into();
            self.nodes.push(filter);
        }

        Ok(last_node)
    }

    pub fn update_params(&self) {
        for (node, spec) in self.nodes.iter().zip(self.specs.iter().filter(|s| s.enabled)) {
            node.frequency().set_value(spec.frequency);
            node.q().set_value(spec.q);
        }
    }

    pub fn set_lowpass(&mut self, freq: f32, q: f32) {
        if let Some(spec) = self.specs.first_mut() {
            spec.kind = FilterKind::Lowpass;
            spec.frequency = freq;
            spec.q = q;
            spec.enabled = true;
        }
        self.update_params();
    }
}
