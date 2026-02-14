# CLAUDE.md

## Project overview

Bublik is a browser-based ambient audio generator built with Leptos (Rust WASM framework). It synthesizes audio in real-time using the Web Audio API — all audio generation happens in Rust compiled to WASM, with no JavaScript audio code or third-party audio libraries.

## Architecture

```
src/
├── main.rs              # Entry point, mounts Leptos app
├── app.rs               # Root component, wires audio engine to UI
├── audio/
│   ├── mod.rs           # Module declarations
│   ├── context.rs       # AudioEngine wrapper around Web Audio API AudioContext
│   ├── sources.rs       # SoundSource: noise generators (ScriptProcessorNode) and oscillators
│   ├── filters.rs       # FilterChain with biquad filters (lowpass, highpass, bandpass, notch)
│   ├── binaural.rs      # BinauralBeat: stereo-split oscillators via ChannelMergerNode
│   └── modulation.rs    # LFO: oscillator connected to AudioParam for modulation
├── ui/
│   ├── mod.rs           # Module declarations
│   ├── terrain.rs       # Canvas rendering: background, rings, orbs, connection lines
│   ├── orb.rs           # OrbData: position, radius, color, hit-testing for draggable orbs
│   └── dock.rs          # Bottom dock: play/pause, volume, source buttons, presets, Me/About dropdowns
├── state/
│   ├── mod.rs           # Module declarations
│   ├── presets.rs       # Built-in preset definitions (Deep Focus, Sleep Descent, etc.)
│   └── persistence.rs   # LocalStorage save/load and URL state encoding
style/
└── main.css             # All CSS: dark theme, dock, pills, dropdowns, mobile responsive
```

## Key patterns

- **Audio types are `!Send + !Sync`** — web-sys types contain raw pointers. Use `StoredValue::new_local()` with `LocalStorage` instead of the default `SyncStorage`.
- **Noise generators** use `ScriptProcessorNode` with `Closure<dyn FnMut(AudioProcessingEvent)>`. Closures must be stored (in `_closures` vec) to prevent garbage collection.
- **Canvas rendering** runs at ~30fps via `setInterval`. The `Effect` system triggers redraws when `anim_time` signal updates.
- **Leptos 0.7 CSR-only** — no SSR, no hydration. The app is mounted directly with `leptos::mount::mount_to_body`.

## Build commands

```sh
# Check compilation (fast)
cargo check --target wasm32-unknown-unknown

# Dev server with hot reload
trunk serve

# Production build
trunk build --release

# Production build for GitHub Pages (with base path)
trunk build --release --public-url /bublik/
```

## Dependencies

- `leptos 0.7` with `csr` feature
- `web-sys 0.3` with extensive Audio/Canvas/DOM features enabled in Cargo.toml
- `wasm-bindgen`, `js-sys` for JS interop
- `serde`, `serde_json` for preset serialization
- `gloo-timers` (available but not heavily used — intervals use raw web-sys)

## Important notes

- The `Cargo.toml` web-sys features list is critical — missing a feature causes compile errors with unhelpful messages. If you add new Web API usage, add the corresponding feature.
- `copy_to_channel` takes `i32` (not `u32`) for the channel index — cast with `ch as i32`.
- All `Closure` instances that are passed to JS event handlers must either be `.forget()`-ed or stored to prevent premature deallocation.
- The `#![allow(dead_code)]` in `main.rs` suppresses warnings for public API items (binaural setters, persistence functions, etc.) that form the available API surface but aren't all called from the current UI.
