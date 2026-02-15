use leptos::prelude::*;

use crate::audio::sources::SourceKind;
use crate::state::presets::Preset;

#[derive(Clone, Debug)]
pub enum DockAction {
    AddSource(SourceKind),
    LoadPreset(usize),
    TogglePlay,
    RemoveLast,
    ToggleBinaural,
}

#[component]
pub fn Dock(
    presets: Vec<Preset>,
    is_playing: ReadSignal<bool>,
    on_action: Callback<DockAction>,
    binaural_active: ReadSignal<bool>,
    master_volume: ReadSignal<f32>,
    on_volume: Callback<f32>,
    active_preset: ReadSignal<Option<usize>>,
) -> impl IntoView {
    let source_kinds = vec![
        ("Brown", SourceKind::BrownNoise),
        ("Pink", SourceKind::PinkNoise),
        ("White", SourceKind::WhiteNoise),
        ("Theta", SourceKind::Theta),
        ("Alpha", SourceKind::Alpha),
        ("Delta", SourceKind::Delta),
        ("432Hz", SourceKind::CustomOsc(432.0)),
        ("Rain", SourceKind::RainTexture),
        ("Harmonic", SourceKind::Harmonic(220.0)),
    ];

    let preset_names: Vec<String> = presets.iter().map(|p| p.name.clone()).collect();

    let (show_about, set_show_about) = signal(false);

    view! {
        <div class="dock" role="toolbar" aria-label="Audio controls">
            <div class="dock-section">
                <button
                    class="dock-btn play-btn"
                    aria-label="Toggle play/pause (Space)"
                    title="Play/Pause (Space)"
                    on:click={
                        let on_action = on_action.clone();
                        move |_| on_action.run(DockAction::TogglePlay)
                    }
                >
                    {move || if is_playing.get() { "\u{23F8}" } else { "\u{25B6}" }}
                </button>

                <div class="volume-control">
                    <span class="vol-label">"Vol"</span>
                    <input
                        type="range"
                        min="0"
                        max="100"
                        aria-label="Master volume (Scroll to adjust)"
                        title="Volume (Scroll)"
                        prop:value={move || (master_volume.get() * 100.0) as i32}
                        on:input={
                            let on_volume = on_volume.clone();
                            move |ev| {
                                use leptos::prelude::*;
                                let val: f32 = event_target_value(&ev).parse().unwrap_or(70.0);
                                on_volume.run(val / 100.0);
                            }
                        }
                        class="vol-slider"
                    />
                </div>
            </div>

            <div class="dock-section sources-section">
                <span class="dock-label">"Add:"</span>
                {source_kinds
                    .into_iter()
                    .map(|(label, kind)| {
                        let on_action = on_action.clone();
                        let aria = format!("Add {} source", label);
                        view! {
                            <button
                                class="dock-pill source-pill"
                                style:border-color={kind.color()}
                                aria-label=aria
                                on:click=move |_| on_action.run(DockAction::AddSource(kind))
                            >
                                {label}
                            </button>
                        }
                    })
                    .collect::<Vec<_>>()}
                <button
                    class="dock-pill remove-pill"
                    aria-label="Remove last source"
                    on:click={
                        let on_action = on_action.clone();
                        move |_| on_action.run(DockAction::RemoveLast)
                    }
                >
                    "\u{2715} Remove"
                </button>
            </div>

            <div class="dock-section">
                <button
                    class="dock-pill binaural-pill"
                    class:active={move || binaural_active.get()}
                    aria-label="Toggle binaural beats"
                    aria-pressed={move || binaural_active.get().to_string()}
                    on:click={
                        let on_action = on_action.clone();
                        move |_| on_action.run(DockAction::ToggleBinaural)
                    }
                >
                    {move || {
                        if binaural_active.get() {
                            "Binaural: ON"
                        } else {
                            "Binaural: OFF"
                        }
                    }}
                </button>
            </div>

            <div class="dock-section presets-section">
                <span class="dock-label">"Presets:"</span>
                {preset_names
                    .into_iter()
                    .enumerate()
                    .map(|(i, name)| {
                        let on_action = on_action.clone();
                        let aria = format!("Load {} preset", name);
                        view! {
                            <button
                                class="dock-pill preset-pill"
                                class:active={move || active_preset.get() == Some(i)}
                                aria-label=aria
                                on:click=move |_| on_action.run(DockAction::LoadPreset(i))
                            >
                                {name}
                            </button>
                        }
                    })
                    .collect::<Vec<_>>()}
            </div>

            <div class="dock-section dock-right">
                <div class="dock-dropdown">
                    <button class="dock-pill me-pill" aria-label="Author links">"Me"</button>
                    <div class="dock-dropdown-content">
                        <a href="https://undivisible.dev" target="_blank" rel="noopener">"undivisible.dev"</a>
                        <a href="https://atechnology.company" target="_blank" rel="noopener">"atechnology.company"</a>
                    </div>
                </div>

                <div class="dock-dropdown">
                    <button
                        class="dock-pill about-pill"
                        aria-label="About Bublik"
                        on:click=move |_| set_show_about.set(!show_about.get_untracked())
                    >
                        "About"
                    </button>
                    <div class="dock-dropdown-content about-content">
                        <div class="about-text">
                            <p class="about-title">"bublik"</p>
                            <p>"A frequency terrain audio generator built with Leptos + Rust WASM. \
                                All audio synthesis runs natively in the browser via the Web Audio API \u{2014} \
                                no third-party audio libraries."</p>
                            <p class="about-features">
                                "Brown/pink/white noise \u{00B7} Theta/alpha/delta waves \u{00B7} \
                                 Binaural beats \u{00B7} Harmonic series \u{00B7} Rain textures \u{00B7} \
                                 Biquad filters \u{00B7} LFO modulation"
                            </p>
                            <p class="about-hint">
                                "Space = play/pause \u{00B7} Scroll = volume \u{00B7} \
                                 Drag orbs to shape sound \u{00B7} Hover orbs for filters"
                            </p>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
