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

    pub fn color(self) -> &'static str {
        match self {
            Self::Lowpass => "#FFA500",
            Self::Highpass => "#4169E1",
            Self::Bandpass => "#00FF88",
            Self::Notch => "#FF00FF",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Lowpass => "LP",
            Self::Highpass => "HP",
            Self::Bandpass => "BP",
            Self::Notch => "Notch",
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
            frequency: 20000.0,
            q: std::f32::consts::FRAC_1_SQRT_2,
            enabled: true,
        }
    }
}

pub struct FilterChain {
    pub specs: Vec<FilterSpec>,
    nodes: Vec<BiquadFilterNode>,
    last_output: Option<web_sys::AudioNode>,
}

impl FilterChain {
    pub fn new() -> Self {
        Self {
            specs: vec![FilterSpec::default()],
            nodes: Vec::new(),
            last_output: None,
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
            last_output: None,
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
            self.last_output = Some(node.clone());
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

        self.last_output = Some(last_node.clone());
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

    pub fn set_filter(&mut self, kind: FilterKind, freq: f32, q: f32) {
        if let Some(spec) = self.specs.first_mut() {
            spec.kind = kind;
            spec.frequency = freq;
            spec.q = q;
            spec.enabled = true;
        }
        if let Some(node) = self.nodes.first() {
            node.set_type(kind.to_biquad_type());
            node.frequency().set_value(freq);
            node.q().set_value(q);
        }
    }

    pub fn add_filter(&mut self, kind: FilterKind, freq: f32, q: f32) {
        self.specs.push(FilterSpec {
            kind,
            frequency: freq,
            q,
            enabled: true,
        });
        // Node will be created on next rebuild; update existing nodes for now
        if let Some(node) = self.nodes.last() {
            // If we already have a node for this spec index, update it
            // Otherwise it will be picked up on rebuild
            let _ = node;
        }
    }

    /// Rebuilds the filter chain, disconnecting old nodes and creating new ones.
    /// Should be called after adding/removing filters.
    pub fn rebuild(
        &mut self,
        ctx: &AudioContext,
        input: &GainNode,
        master: &web_sys::AudioNode,
    ) -> Result<(), JsValue> {
        // Disconnect the last output from master
        if let Some(ref old_output) = self.last_output {
            let _ = old_output.disconnect();
        }
        
        // Disconnect all old filter nodes
        for node in &self.nodes {
            let _ = node.disconnect();
        }
        
        // Build new chain and connect to master
        let output = self.build(ctx, input)?;
        output.connect_with_audio_node(master)?;
        Ok(())
    }

    pub fn remove_filter_by_kind(&mut self, kind: FilterKind) {
        self.specs.retain(|s| s.kind != kind || !s.enabled);
        // Ensure we always have at least the default passthrough filter
        if self.specs.is_empty() {
            self.specs.push(FilterSpec::default());
        }
        // Note: The audio chain needs to be rebuilt after this call
        // by calling rebuild() with the audio context
    }

    pub fn has_filter(&self, kind: FilterKind) -> bool {
        self.specs.iter().any(|s| s.kind == kind && s.enabled && s.frequency < 19000.0)
    }

    pub fn clear_filter(&mut self) {
        // Reset to single passthrough lowpass
        self.specs.clear();
        self.specs.push(FilterSpec::default());
        for node in &self.nodes {
            node.set_type(BiquadFilterType::Lowpass);
            node.frequency().set_value(20000.0);
            node.q().set_value(std::f32::consts::FRAC_1_SQRT_2);
        }
    }

    pub fn set_cutoff(&mut self, freq: f32) {
        // Update all lowpass and bandpass filters
        for (spec, node) in self.specs.iter_mut().zip(self.nodes.iter()) {
            if matches!(spec.kind, FilterKind::Lowpass | FilterKind::Bandpass) {
                spec.frequency = freq;
                node.frequency().set_value(freq);
            }
        }
    }

    pub fn active_kind(&self) -> Option<FilterKind> {
        self.specs.first().and_then(|s| {
            if s.enabled && s.frequency < 19000.0 {
                Some(s.kind)
            } else {
                None
            }
        })
    }

    pub fn active_kinds(&self) -> Vec<FilterKind> {
        self.specs
            .iter()
            .filter(|s| s.enabled && s.frequency < 19000.0)
            .map(|s| s.kind)
            .collect()
    }
}

/// Logarithmic mapping from normalized x (0-1) to filter cutoff (20-20000 Hz)
pub fn x_to_cutoff(norm_x: f32) -> f32 {
    20.0 * (1000.0_f32).powf(norm_x)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_kind_color_valid_hex() {
        let kinds = [FilterKind::Lowpass, FilterKind::Highpass, FilterKind::Bandpass, FilterKind::Notch];
        for kind in &kinds {
            let color = kind.color();
            assert!(color.starts_with('#'));
            assert_eq!(color.len(), 7);
        }
    }

    #[test]
    fn test_filter_kind_label() {
        assert_eq!(FilterKind::Lowpass.label(), "LP");
        assert_eq!(FilterKind::Highpass.label(), "HP");
        assert_eq!(FilterKind::Bandpass.label(), "BP");
        assert_eq!(FilterKind::Notch.label(), "Notch");
    }

    #[test]
    fn test_filter_kind_serialization_roundtrip() {
        let kinds = [FilterKind::Lowpass, FilterKind::Highpass, FilterKind::Bandpass, FilterKind::Notch];
        for kind in &kinds {
            let json = serde_json::to_string(kind).unwrap();
            let deserialized: FilterKind = serde_json::from_str(&json).unwrap();
            assert_eq!(*kind, deserialized);
        }
    }

    #[test]
    fn test_filter_spec_default() {
        let spec = FilterSpec::default();
        assert_eq!(spec.kind, FilterKind::Lowpass);
        assert_eq!(spec.frequency, 20000.0);
        assert!(spec.enabled);
    }

    #[test]
    fn test_filter_chain_new() {
        let chain = FilterChain::new();
        assert_eq!(chain.specs.len(), 1);
        assert_eq!(chain.specs[0].kind, FilterKind::Lowpass);
        assert_eq!(chain.specs[0].frequency, 20000.0);
        assert!(chain.specs[0].enabled);
    }

    #[test]
    fn test_filter_chain_active_kind_passthrough() {
        let chain = FilterChain::new();
        // Default 20kHz is passthrough, should return None
        assert!(chain.active_kind().is_none());
    }

    #[test]
    fn test_x_to_cutoff_range() {
        let min = x_to_cutoff(0.0);
        let mid = x_to_cutoff(0.5);
        let max = x_to_cutoff(1.0);

        assert!((min - 20.0).abs() < 0.1, "x=0 should be ~20Hz, got {}", min);
        assert!(mid > 400.0 && mid < 700.0, "x=0.5 should be ~632Hz, got {}", mid);
        assert!((max - 20000.0).abs() < 1.0, "x=1 should be ~20000Hz, got {}", max);

        // Should be monotonically increasing
        assert!(x_to_cutoff(0.25) < x_to_cutoff(0.5));
        assert!(x_to_cutoff(0.5) < x_to_cutoff(0.75));
    }

    #[test]
    fn test_filter_chain_with_lowpass() {
        let chain = FilterChain::with_lowpass(1000.0, 1.0);
        assert_eq!(chain.specs[0].kind, FilterKind::Lowpass);
        assert_eq!(chain.specs[0].frequency, 1000.0);
        assert_eq!(chain.specs[0].q, 1.0);
        assert!(chain.specs[0].enabled);
        assert_eq!(chain.active_kind(), Some(FilterKind::Lowpass));
    }
}
