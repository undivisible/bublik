use crate::audio::filters::FilterKind;
use crate::audio::sources::SourceKind;

#[derive(Clone, Debug)]
pub struct OrbData {
    pub id: usize,
    pub kind: SourceKind,
    pub x: f64,
    pub y: f64,
    pub norm_x: f32,
    pub norm_y: f32,
    pub radius: f64,
    pub gain: f32,
    pub freq: f32,
    pub color: String,
    /// Pre-computed RGB from color hex
    pub rgb: (u8, u8, u8),
    pub pulse_phase: f64,
    pub binaural_active: bool,
    pub active_filters: Vec<FilterKind>,
}

impl OrbData {
    pub fn new(id: usize, kind: SourceKind, x: f64, y: f64, norm_x: f32, norm_y: f32) -> Self {
        let active_filters = if kind.is_noise() {
            vec![FilterKind::Lowpass]
        } else {
            vec![]
        };
        let color = kind.color().to_string();
        let rgb = hex_to_rgb(&color);
        Self {
            id,
            kind,
            x,
            y,
            norm_x,
            norm_y,
            radius: 20.0 + norm_y as f64 * 30.0,
            gain: norm_y,
            freq: kind.default_freq(),
            color,
            rgb,
            pulse_phase: js_sys::Math::random() * std::f64::consts::TAU,
            binaural_active: false,
            active_filters,
        }
    }

    pub fn contains(&self, px: f64, py: f64) -> bool {
        let dx = px - self.x;
        let dy = py - self.y;
        (dx * dx + dy * dy).sqrt() <= self.radius + 10.0
    }

    pub fn update_radius(&mut self) {
        self.radius = 20.0 + self.gain as f64 * 30.0;
    }
}

pub fn hex_to_rgb(hex: &str) -> (u8, u8, u8) {
    if hex.len() < 7 {
        return (128, 128, 128);
    }
    let r = u8::from_str_radix(&hex[1..3], 16).unwrap_or(128);
    let g = u8::from_str_radix(&hex[3..5], 16).unwrap_or(128);
    let b = u8::from_str_radix(&hex[5..7], 16).unwrap_or(128);
    (r, g, b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_to_rgb() {
        assert_eq!(hex_to_rgb("#FF0000"), (255, 0, 0));
        assert_eq!(hex_to_rgb("#00FF00"), (0, 255, 0));
        assert_eq!(hex_to_rgb("#0000FF"), (0, 0, 255));
        assert_eq!(hex_to_rgb("#8B4513"), (139, 69, 19));
        assert_eq!(hex_to_rgb("#FFA500"), (255, 165, 0));
    }

    #[test]
    fn test_hex_to_rgb_invalid() {
        assert_eq!(hex_to_rgb("short"), (128, 128, 128));
        assert_eq!(hex_to_rgb(""), (128, 128, 128));
    }

    #[test]
    fn test_orb_contains_inside() {
        let orb = OrbData {
            id: 0,
            kind: SourceKind::BrownNoise,
            x: 100.0,
            y: 100.0,
            norm_x: 0.5,
            norm_y: 0.5,
            radius: 30.0,
            gain: 0.5,
            freq: 0.0,
            color: "#8B4513".to_string(),
            rgb: (139, 69, 19),
            pulse_phase: 0.0,
            binaural_active: false,
            active_filters: vec![],
        };
        // Inside radius + 10
        assert!(orb.contains(100.0, 100.0)); // center
        assert!(orb.contains(130.0, 100.0)); // at radius
        assert!(orb.contains(139.0, 100.0)); // within +10 margin
        // Outside
        assert!(!orb.contains(141.0, 100.0)); // past margin
        assert!(!orb.contains(200.0, 200.0)); // far away
    }

    #[test]
    fn test_update_radius() {
        let mut orb = OrbData {
            id: 0,
            kind: SourceKind::BrownNoise,
            x: 0.0,
            y: 0.0,
            norm_x: 0.0,
            norm_y: 0.0,
            radius: 20.0,
            gain: 0.0,
            freq: 0.0,
            color: "#000000".to_string(),
            rgb: (0, 0, 0),
            pulse_phase: 0.0,
            binaural_active: false,
            active_filters: vec![],
        };
        assert_eq!(orb.radius, 20.0);
        orb.gain = 1.0;
        orb.update_radius();
        assert_eq!(orb.radius, 50.0); // 20 + 1.0 * 30
        orb.gain = 0.5;
        orb.update_radius();
        assert_eq!(orb.radius, 35.0); // 20 + 0.5 * 30
    }
}
