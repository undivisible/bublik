use crate::audio::filters::FilterKind;
use crate::audio::sources::SourceKind;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PresetSource {
    pub kind: SourceKind,
    pub gain: f32,
    pub x: f32,
    pub y: f32,
    pub filter_kind: Option<FilterKind>,
    pub filter_freq: Option<f32>,
    pub filter_q: Option<f32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PresetBinaural {
    pub base_freq: f32,
    pub beat_freq: f32,
    pub gain: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Preset {
    pub name: String,
    pub sources: Vec<PresetSource>,
    pub binaural: Option<PresetBinaural>,
    pub master_volume: f32,
}

pub fn built_in_presets() -> Vec<Preset> {
    vec![
        Preset {
            name: "Deep Focus".into(),
            sources: vec![PresetSource {
                kind: SourceKind::BrownNoise,
                gain: 0.5,
                x: 0.3,
                y: 0.5,
                filter_kind: Some(FilterKind::Lowpass),
                filter_freq: Some(800.0),
                filter_q: Some(1.0),
            }],
            binaural: Some(PresetBinaural {
                base_freq: 200.0,
                beat_freq: 6.0,
                gain: 0.4,
            }),
            master_volume: 0.7,
        },
        Preset {
            name: "Sleep Descent".into(),
            sources: vec![PresetSource {
                kind: SourceKind::BrownNoise,
                gain: 0.6,
                x: 0.2,
                y: 0.6,
                filter_kind: Some(FilterKind::Lowpass),
                filter_freq: Some(400.0),
                filter_q: Some(0.7),
            }],
            binaural: Some(PresetBinaural {
                base_freq: 180.0,
                beat_freq: 4.0,
                gain: 0.3,
            }),
            master_volume: 0.6,
        },
        Preset {
            name: "Meditation".into(),
            sources: vec![
                PresetSource {
                    kind: SourceKind::Theta,
                    gain: 0.3,
                    x: 0.5,
                    y: 0.3,
                    filter_kind: None,
                    filter_freq: None,
                    filter_q: None,
                },
                PresetSource {
                    kind: SourceKind::CustomOsc(432.0),
                    gain: 0.15,
                    x: 0.7,
                    y: 0.15,
                    filter_kind: None,
                    filter_freq: None,
                    filter_q: None,
                },
            ],
            binaural: Some(PresetBinaural {
                base_freq: 216.0,
                beat_freq: 6.0,
                gain: 0.35,
            }),
            master_volume: 0.6,
        },
        Preset {
            name: "Rain Café".into(),
            sources: vec![
                PresetSource {
                    kind: SourceKind::RainTexture,
                    gain: 0.5,
                    x: 0.4,
                    y: 0.5,
                    filter_kind: None,
                    filter_freq: None,
                    filter_q: None,
                },
                PresetSource {
                    kind: SourceKind::PinkNoise,
                    gain: 0.2,
                    x: 0.6,
                    y: 0.2,
                    filter_kind: Some(FilterKind::Bandpass),
                    filter_freq: Some(800.0),
                    filter_q: Some(2.0),
                },
            ],
            binaural: Some(PresetBinaural {
                base_freq: 200.0,
                beat_freq: 10.0,
                gain: 0.2,
            }),
            master_volume: 0.7,
        },
        Preset {
            name: "Void".into(),
            sources: vec![
                PresetSource {
                    kind: SourceKind::BrownNoise,
                    gain: 0.7,
                    x: 0.15,
                    y: 0.7,
                    filter_kind: Some(FilterKind::Lowpass),
                    filter_freq: Some(200.0),
                    filter_q: Some(1.0),
                },
                PresetSource {
                    kind: SourceKind::CustomOsc(40.0),
                    gain: 0.3,
                    x: 0.1,
                    y: 0.3,
                    filter_kind: None,
                    filter_freq: None,
                    filter_q: None,
                },
            ],
            binaural: Some(PresetBinaural {
                base_freq: 150.0,
                beat_freq: 4.0,
                gain: 0.25,
            }),
            master_volume: 0.6,
        },
    ]
}
