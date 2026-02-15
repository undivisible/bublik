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
        <div
            class="fixed bottom-0 left-0 right-0 flex items-center gap-2.5 px-4 pt-2.5 pb-[calc(10px+env(safe-area-inset-bottom,0px))] bg-[rgba(10,10,10,0.94)] backdrop-blur-[20px] border-t border-white/[0.06] overflow-x-auto no-scrollbar z-[100] flex-nowrap md:flex-wrap md:justify-center"
            role="toolbar"
            aria-label="Audio controls"
        >
            // Play + Volume
            <div class="flex items-center gap-1.5 shrink-0">
                <button
                    class="bg-transparent border border-white/[0.12] text-gray-200 w-[42px] h-[42px] rounded-full cursor-pointer text-base flex items-center justify-center transition-all duration-250 hover:border-cyan-400 hover:text-cyan-400 hover:shadow-[0_0_16px_rgba(0,206,209,0.35)] hover:scale-[1.08]"
                    aria-label="Toggle play/pause (Space)"
                    title="Play/Pause (Space)"
                    on:click={
                        let on_action = on_action.clone();
                        move |_| on_action.run(DockAction::TogglePlay)
                    }
                >
                    {move || if is_playing.get() {
                        view! {
                            <svg viewBox="0 0 24 24" width="18" height="18" fill="currentColor">
                                <path d="M6 4h4v16H6zm8 0h4v16h-4z" />
                            </svg>
                        }.into_any()
                    } else {
                        view! {
                            <svg viewBox="0 0 24 24" width="18" height="18" fill="currentColor">
                                <path d="M8 5v14l11-7z" />
                            </svg>
                        }.into_any()
                    }}
                </button>

                <div class="flex items-center gap-1.5">
                    <span class="text-[8px] text-white/25 uppercase tracking-[1px]">"Vol"</span>
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

            // Sources
            <div class="flex items-center gap-1.5 shrink-0 flex-wrap justify-center max-w-full md:max-w-none">
                <span class="text-[9px] text-white/30 uppercase tracking-[1.5px] font-medium">"Add:"</span>
                {source_kinds
                    .into_iter()
                    .map(|(label, kind)| {
                        let on_action = on_action.clone();
                        let aria = format!("Add {} source", label);
                        view! {
                            <button
                                class="bg-white/[0.03] border border-white/[0.08] text-[#b0b0b0] px-2.5 py-[5px] rounded-2xl text-[11px] font-mono cursor-pointer transition-all duration-200 whitespace-nowrap hover:bg-white/[0.07] hover:border-white/20 hover:text-white hover:-translate-y-px hover:shadow-[0_0_12px_rgba(255,255,255,0.08)]"
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
                    class="bg-white/[0.03] border border-[rgba(255,80,80,0.25)] text-[rgba(255,80,80,0.6)] px-2.5 py-[5px] rounded-2xl text-[11px] font-mono cursor-pointer transition-all duration-200 whitespace-nowrap hover:border-[rgba(255,80,80,0.5)] hover:text-[#ff5050] hover:bg-[rgba(255,80,80,0.06)] hover:shadow-[0_0_12px_rgba(255,80,80,0.15)]"
                    aria-label="Remove last source"
                    on:click={
                        let on_action = on_action.clone();
                        move |_| on_action.run(DockAction::RemoveLast)
                    }
                >
                    "\u{2715} Remove"
                </button>
            </div>

            // Binaural
            <div class="flex items-center gap-1.5 shrink-0">
                <button
                    class="px-2.5 py-[5px] rounded-2xl text-[11px] font-mono cursor-pointer transition-all duration-200 whitespace-nowrap border"
                    style:background=move || if binaural_active.get() { "rgba(0,206,209,0.1)" } else { "rgba(255,255,255,0.03)" }
                    style:border-color=move || if binaural_active.get() { "#00CED1" } else { "rgba(0,206,209,0.25)" }
                    style:color=move || if binaural_active.get() { "#00CED1" } else { "#b0b0b0" }
                    style:box-shadow=move || if binaural_active.get() { "0 0 12px rgba(0,206,209,0.25)" } else { "none" }
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

            // Presets
            <div class="flex items-center gap-1.5 shrink-0 ml-0 md:ml-auto">
                <span class="text-[9px] text-white/30 uppercase tracking-[1.5px] font-medium">"Presets:"</span>
                {preset_names
                    .into_iter()
                    .enumerate()
                    .map(|(i, name)| {
                        let on_action = on_action.clone();
                        let aria = format!("Load {} preset", name);
                        view! {
                            <button
                                class="px-2.5 py-[5px] rounded-2xl text-[11px] font-mono cursor-pointer transition-all duration-200 whitespace-nowrap border"
                                style:background=move || if active_preset.get() == Some(i) { "rgba(147,112,219,0.12)" } else { "rgba(255,255,255,0.03)" }
                                style:border-color=move || if active_preset.get() == Some(i) { "#9370db" } else { "rgba(147,112,219,0.25)" }
                                style:color=move || if active_preset.get() == Some(i) { "#9370db" } else { "#b0b0b0" }
                                style:box-shadow=move || if active_preset.get() == Some(i) { "0 0 14px rgba(147,112,219,0.3)" } else { "none" }
                                aria-label=aria
                                on:click=move |_| on_action.run(DockAction::LoadPreset(i))
                            >
                                {name}
                            </button>
                        }
                    })
                    .collect::<Vec<_>>()}
            </div>

            // Me + About
            <div class="flex items-center gap-1.5 shrink-0 ml-0 md:ml-auto">
                <div class="group relative">
                    <button
                        class="bg-white/[0.03] border border-[rgba(0,255,136,0.25)] text-[#b0b0b0] px-2.5 py-[5px] rounded-2xl text-[11px] font-mono cursor-pointer transition-all duration-200 whitespace-nowrap hover:border-[rgba(0,255,136,0.5)] hover:text-[#00ff88] hover:shadow-[0_0_12px_rgba(0,255,136,0.2)]"
                        aria-label="Author links"
                    >
                        "Me"
                    </button>
                    <div class="hidden group-hover:block animate-slide-up absolute bottom-[calc(100%+8px)] right-0 min-w-[180px] bg-[rgba(14,14,14,0.97)] backdrop-blur-[20px] border border-white/[0.08] rounded-xl py-1.5 z-[200] shadow-[0_-8px_32px_rgba(0,0,0,0.6)] before:content-[''] before:absolute before:top-full before:left-0 before:right-0 before:h-2.5 before:bg-transparent">
                        <a
                            href="https://undivisible.dev"
                            target="_blank"
                            rel="noopener"
                            class="block px-4 py-2 text-[#a0a0a0] no-underline text-xs font-mono transition-all duration-150 hover:text-cyan-400 hover:bg-[rgba(0,206,209,0.06)]"
                        >
                            "undivisible.dev"
                        </a>
                        <a
                            href="https://atechnology.company"
                            target="_blank"
                            rel="noopener"
                            class="block px-4 py-2 text-[#a0a0a0] no-underline text-xs font-mono transition-all duration-150 hover:text-cyan-400 hover:bg-[rgba(0,206,209,0.06)]"
                        >
                            "atechnology.company"
                        </a>
                    </div>
                </div>

                <div class="group relative">
                    <button
                        class="bg-white/[0.03] border border-[rgba(255,215,0,0.25)] text-[#b0b0b0] px-2.5 py-[5px] rounded-2xl text-[11px] font-mono cursor-pointer transition-all duration-200 whitespace-nowrap hover:border-[rgba(255,215,0,0.5)] hover:text-[#ffd700] hover:shadow-[0_0_12px_rgba(255,215,0,0.2)]"
                        aria-label="About Bublik"
                        on:click=move |_| set_show_about.set(!show_about.get_untracked())
                    >
                        "About"
                    </button>
                    <div class="hidden group-hover:block animate-slide-up absolute bottom-[calc(100%+8px)] right-0 min-w-[300px] bg-[rgba(14,14,14,0.97)] backdrop-blur-[20px] border border-white/[0.08] rounded-xl z-[200] shadow-[0_-8px_32px_rgba(0,0,0,0.6)] before:content-[''] before:absolute before:top-full before:left-0 before:right-0 before:h-2.5 before:bg-transparent">
                        <div class="px-5 py-4 text-[11px] leading-[1.7] text-white/55">
                            <p class="text-[15px] font-semibold text-cyan-400 mb-2 tracking-[2px] lowercase">"bublik"</p>
                            <p>"A frequency terrain audio generator built with Leptos + Rust WASM. \
                                All audio synthesis runs natively in the browser via the Web Audio API \u{2014} \
                                no third-party audio libraries."</p>
                            <p class="mt-2.5 text-[10px] text-white/35 leading-[1.8]">
                                "Brown/pink/white noise \u{00B7} Theta/alpha/delta waves \u{00B7} \
                                 Binaural beats \u{00B7} Harmonic series \u{00B7} Rain textures \u{00B7} \
                                 Biquad filters \u{00B7} LFO modulation"
                            </p>
                            <p class="mt-3 text-[10px] text-cyan-400/45 italic">
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
