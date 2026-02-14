use crate::audio::sources::SourceKind;

#[derive(Clone, Debug)]
pub struct OrbData {
    pub id: usize,
    pub kind: SourceKind,
    pub x: f64,
    pub y: f64,
    pub radius: f64,
    pub gain: f32,
    pub freq: f32,
    pub color: String,
    pub pulse_phase: f64,
}

impl OrbData {
    pub fn new(id: usize, kind: SourceKind, x: f64, y: f64, gain: f32) -> Self {
        Self {
            id,
            kind,
            x,
            y,
            radius: 20.0 + gain as f64 * 30.0,
            gain,
            freq: kind.default_freq(),
            color: kind.color().to_string(),
            pulse_phase: 0.0,
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
