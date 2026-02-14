## Prompt: Leptos + Rust Theta/Brown Noise Generator

Build a **web-based audio generator** using **Leptos** (Rust WASM framework) with real-time audio synthesis via Web Audio API.

### Core Audio Engine
All synthesis happens in Rust compiled to WASM, interfacing with Web Audio API via `web-sys`:

1. **Brown noise generator** — Brownian motion algorithm (accumulated random walk, filtered)
2. **Theta wave generator** — Pure sine oscillator (4-8 Hz) OR binaural beat mode (two slightly offset frequencies, one per ear)
3. **Mixer** — Layer unlimited sound sources with individual gain
4. **Filter chain** — Biquad filters (lowpass, highpass, bandpass, notch) with real-time parameter control
5. **Modulation** — LFO that can modulate any parameter (volume, filter cutoff, pan, pitch)

### Sound Sources (user can add/remove/stack infinitely)
- Brown noise (with color slider: pink ↔ brown ↔ red)
- White/pink noise
- Theta oscillator (sine, 4-8 Hz) — direct or binaural
- Alpha oscillator (8-13 Hz)
- Delta oscillator (0.5-4 Hz)
- Custom frequency oscillator (0.1 Hz - 20 kHz)
- Harmonic series generator (fundamental + overtones)
- Rain/water texture (filtered noise with resonant peaks)

### Filters Per Source
Each source gets its own filter chain:
- **Lowpass/Highpass** with cutoff + resonance (Q)
- **Bandpass** for isolating frequency ranges
- **Notch** for removing annoying frequencies
- **Comb filter** for metallic/resonant textures
- **Reverb** (convolution or algorithmic)
- **Stereo width** control

### Binaural Beat Engine
- **Base frequency** slider (100-500 Hz carrier)
- **Beat frequency** slider (0.5-40 Hz, with labeled zones: delta/theta/alpha/beta/gamma)
- **Isochronal mode** — pulsing tone instead of binaural (works on speakers, not just headphones)
- Auto-ramp: slowly shift beat frequency over time (e.g., 10 Hz → 4 Hz over 30 min for sleep induction)

### Unorthodox Design
**Concept: "Frequency Terrain"** — NOT a typical mixer/DAW layout.

- **Dark background** (#0a0a0a) with bioluminescent accent colors (deep teal, amber, soft violet)
- **No traditional sliders.** Instead:
  - **Draggable orbs** on a 2D canvas — X axis = frequency, Y axis = amplitude
  - Each sound source is an orb with a glowing aura (size = volume, color = type)
  - Drag orbs around to change parameters fluidly
  - Orbs pulse subtly at their own frequency
- **Concentric rings** radiating from center = frequency bands (like a radar/sonar display)
- **Filter controls** appear as translucent arcs around each orb — drag arc edges to set cutoff
- **Connection lines** between orbs = modulation routing (drag from one orb to another to modulate)
- **Breathing background** — the entire UI subtly pulses at the dominant theta frequency
- **Minimal text** — mostly visual/spatial interaction
- **Bottom dock**: tiny preset pills, timer, save/load
- **No visible borders, cards, or boxes** — everything floats

### Technical Stack
- **Leptos 0.7+** with SSR disabled (CSR only, it's an audio app)
- **web-sys** for Web Audio API (`AudioContext`, `OscillatorNode`, `BiquadFilterNode`, `GainNode`, `StereoPannerNode`, `ChannelSplitterNode`, `ChannelMergerNode`)
- **Canvas/WebGL** for the orb visualization (use `web-sys` canvas or `leptos-canvas`)
- **Tailwind CSS** for the minimal UI chrome (dock, modals)
- **LocalStorage** for saving presets
- **No backend needed** — pure client-side WASM

### Presets
Include these built-in presets:
- **Deep Focus** — brown noise + 6 Hz theta binaural, lowpass at 800 Hz
- **Sleep Descent** — brown noise + auto-ramp 8 Hz → 2 Hz over 45 min, heavy reverb
- **Meditation** — pure 6 Hz theta + 432 Hz drone, minimal noise
- **Rain Café** — layered pink noise + resonant peaks at 200/800/2000 Hz, gentle 10 Hz alpha
- **Void** — deep brown noise, lowpass 200 Hz, sub-bass 40 Hz drone, 4 Hz delta

### Key Features
- **Timer** with gentle fade-out
- **Session recording** — export as WAV
- **Keyboard shortcuts** (space = play/pause, scroll = master volume)
- **URL state** — entire configuration encoded in URL hash for sharing
- **Responsive** — works on mobile (touch-drag orbs)
- **Offline capable** — service worker, works without internet

### File Structure
```
src/
├── main.rs              # Leptos app entry
├── app.rs               # Root component
├── audio/
│   ├── mod.rs           # Audio engine coordinator
│   ├── context.rs       # Web Audio API wrapper
│   ├── sources.rs       # Noise/oscillator generators
│   ├── filters.rs       # Filter chain
│   ├── binaural.rs      # Binaural beat engine
│   └── modulation.rs    # LFO / parameter modulation
├── ui/
│   ├── terrain.rs       # Main canvas visualization
│   ├── orb.rs           # Draggable sound source orbs
│   ├── dock.rs          # Bottom preset/control dock
│   └── modal.rs         # Settings/preset modals
├── state/
│   ├── mod.rs           # App state management
│   ├── presets.rs       # Preset definitions
│   └── persistence.rs   # LocalStorage save/load
└── style/
    └── main.css         # Tailwind + custom styles
```

### Constraints
- All audio synthesis in Rust/WASM — no JavaScript audio code
- Smooth real-time parameter changes (no clicks/pops — use `linearRampToValueAtTime`)
- Must handle 10+ simultaneous sources without audio glitching
- Mobile-friendly touch interactions
- Accessible: keyboard navigation for all controls
- Dark mode only (it's an ambient app)

Build the complete app. Start with the audio engine, then the visualization, then presets.

---

### Theta Wave Reference
- 4-6 Hz: deep meditation, hypnagogic states, creativity
- 6-7 Hz: "the zone" — relaxation + alertness, most popular for focus
- 7-8 Hz: light meditation, memory consolidation
- Most popular binaural beat: 6 Hz (e.g., 200 Hz left + 206 Hz right)
