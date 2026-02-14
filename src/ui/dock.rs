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

    view! {
        <div class="dock">
            <div class="dock-section">
                <button
                    class="dock-btn play-btn"
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
                        view! {
                            <button
                                class="dock-pill source-pill"
                                style:border-color={kind.color()}
                                on:click=move |_| on_action.run(DockAction::AddSource(kind))
                            >
                                {label}
                            </button>
                        }
                    })
                    .collect::<Vec<_>>()}
                <button
                    class="dock-pill remove-pill"
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
                        view! {
                            <button
                                class="dock-pill preset-pill"
                                on:click=move |_| on_action.run(DockAction::LoadPreset(i))
                            >
                                {name}
                            </button>
                        }
                    })
                    .collect::<Vec<_>>()}
            </div>
        </div>
    }
}
