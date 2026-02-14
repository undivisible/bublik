# Bublik

A frequency terrain audio generator built with Leptos + Rust WASM. All audio synthesis runs natively in the browser via the Web Audio API — no third-party audio libraries.

## What it does

Bublik generates ambient audio (noise, brainwave frequencies, binaural beats) through an interactive "Frequency Terrain" interface. Sound sources appear as glowing orbs on a dark canvas — drag them to change frequency (X axis) and amplitude (Y axis).

### Sound sources

- **Brown noise** — Brownian motion random walk
- **Pink noise** — Voss-McCartney algorithm
- **White noise** — uniform random
- **Theta oscillator** — 4-8 Hz (deep meditation, focus)
- **Alpha oscillator** — 8-13 Hz (relaxation, alertness)
- **Delta oscillator** — 0.5-4 Hz (deep sleep)
- **Custom frequency** — any frequency 0.1 Hz - 20 kHz
- **Harmonic series** — fundamental + 6 overtones
- **Rain texture** — filtered noise with resonant peaks and random droplets

### Audio features

- **Binaural beats** — stereo-split oscillators with configurable base + beat frequency
- **Filter chain** — biquad filters (lowpass, highpass, bandpass, notch) per source
- **LFO modulation** — sine oscillator routable to any audio parameter

### Built-in presets

- **Deep Focus** — brown noise + 6 Hz theta binaural, lowpass 800 Hz
- **Sleep Descent** — brown noise + delta binaural, heavy lowpass
- **Meditation** — pure theta + 432 Hz drone
- **Rain Cafe** — layered pink noise + rain texture, alpha binaural
- **Void** — deep brown noise, lowpass 200 Hz, sub-bass drone, delta

## Controls

| Action | Control |
|--------|---------|
| Play / Pause | Space bar or play button |
| Master volume | Scroll wheel or volume slider |
| Add source | Click source type in dock |
| Move source | Drag orb on canvas |
| Remove source | Click "Remove" button |
| Binaural beats | Toggle "Binaural" button |
| Load preset | Click preset name in dock |

## Development

### Prerequisites

- Rust (stable)
- `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`
- Trunk: `cargo install trunk`

### Run locally

```sh
trunk serve
```

Opens at `http://localhost:8080`.

### Build for production

```sh
trunk build --release
```

Output goes to `dist/`.

## Tech stack

- **Leptos 0.7** — Rust WASM framework (CSR only)
- **web-sys** — Web Audio API bindings (`AudioContext`, `OscillatorNode`, `BiquadFilterNode`, `GainNode`, `ScriptProcessorNode`, `ChannelMergerNode`)
- **Canvas 2D** — orb visualization with glow effects and pulsing animations
- **No backend** — pure client-side, works offline once loaded

## License

MPL-2.0
