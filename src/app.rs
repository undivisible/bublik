use leptos::prelude::*;
use leptos::reactive::owner::LocalStorage;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, MouseEvent, TouchEvent};

use crate::audio::binaural::BinauralBeat;
use crate::audio::context::AudioEngine;
use crate::audio::filters::{self, FilterKind};
use crate::audio::sources::{SoundSource, SourceKind};
use crate::state::persistence;
use crate::state::presets::{built_in_presets, Preset, PresetBinaural, PresetSource};
use crate::ui::dock::{Dock, DockAction};
use crate::ui::orb::OrbData;
use crate::ui::terrain::draw_terrain;

use filters::x_to_cutoff;

/// Default position for each source kind
fn default_position(kind: SourceKind) -> (f32, f32) {
    match kind {
        SourceKind::BrownNoise => (0.3, 0.5),
        SourceKind::PinkNoise => (0.5, 0.4),
        SourceKind::WhiteNoise => (0.7, 0.3),
        SourceKind::Theta => (0.15, 0.3),
        SourceKind::Alpha => (0.2, 0.3),
        SourceKind::Delta => (0.15, 0.4),
        SourceKind::CustomOsc(f) => ((f / 1000.0).clamp(0.05, 0.95), 0.3),
        SourceKind::Harmonic(f) => ((f / 1000.0).clamp(0.05, 0.95), 0.3),
        SourceKind::RainTexture => (0.4, 0.5),
    }
}

#[component]
pub fn App() -> impl IntoView {
    let (is_playing, set_is_playing) = signal(false);
    let (master_vol, set_master_vol) = signal(0.7f32);
    let (binaural_active, set_binaural_active) = signal(false);
    let (orbs, set_orbs) = signal(Vec::<OrbData>::new());
    let (dragging_id, set_dragging_id) = signal(Option::<usize>::None);
    let (hovered_orb_id, set_hovered_orb_id) = signal(Option::<usize>::None);
    let (is_hovering_menu, set_is_hovering_menu) = signal(false);
    let (active_preset, set_active_preset) = signal(Option::<usize>::None);

    let next_id = StoredValue::new_local(0usize);
    let (anim_time, set_anim_time) = signal(0.0f64);

    let canvas_ref: NodeRef<leptos::html::Canvas> = NodeRef::new();

    let engine: StoredValue<std::cell::RefCell<Option<AudioEngine>>, LocalStorage> =
        StoredValue::new_local(std::cell::RefCell::new(None));
    let binaural_store: StoredValue<std::cell::RefCell<Option<BinauralBeat>>, LocalStorage> =
        StoredValue::new_local(std::cell::RefCell::new(None));

    let ensure_engine = move || {
        engine.with_value(|cell| {
            let mut eng = cell.borrow_mut();
            if eng.is_none() {
                if let Ok(e) = AudioEngine::new() {
                    *eng = Some(e);
                }
            }
        });
    };

    let get_canvas_el = move || -> Option<HtmlCanvasElement> {
        canvas_ref.get().map(|c| {
            let el: &HtmlCanvasElement = &c;
            el.clone()
        })
    };

    // Use clientWidth/clientHeight for 1:1 pixel mapping
    let resize_canvas = move || {
        if let Some(canvas) = get_canvas_el() {
            let el: &web_sys::HtmlElement = canvas.as_ref();
            let w = el.client_width() as u32;
            let h = el.client_height() as u32;
            if w > 0 && h > 0 {
                canvas.set_width(w);
                canvas.set_height(h);
            }
        }
    };

    // Save current session to localStorage
    let save_session = move || {
        let current_orbs = orbs.get_untracked();
        let vol = master_vol.get_untracked();
        let bactive = binaural_active.get_untracked();

        let sources: Vec<PresetSource> = current_orbs
            .iter()
            .map(|orb| {
                // Save first active filter for preset compatibility
                let filter_kind = orb.active_filters.first().copied();
                let (filter_freq, filter_q) = if filter_kind.is_some() {
                    if orb.kind.is_noise() {
                        (Some(x_to_cutoff(orb.norm_x)), Some(1.0))
                    } else {
                        (Some(1000.0), Some(1.0))
                    }
                } else {
                    (None, None)
                };

                PresetSource {
                    kind: orb.kind,
                    gain: orb.gain,
                    x: orb.norm_x,
                    y: orb.norm_y,
                    filter_kind,
                    filter_freq,
                    filter_q,
                }
            })
            .collect();

        let binaural = if bactive {
            Some(PresetBinaural {
                base_freq: 200.0,
                beat_freq: 6.0,
                gain: 0.4,
            })
        } else {
            None
        };

        let session = Preset {
            name: "_session".into(),
            sources,
            binaural,
            master_volume: vol,
        };

        let _ = persistence::save_session(&session);
    };

    // Animation loop (interval-based ~30fps)
    Effect::new(move |_| {
        resize_canvas();

        let interval_cb = Closure::wrap(Box::new(move || {
            let time = web_sys::window()
                .and_then(|w| w.performance())
                .map(|p| p.now() / 1000.0)
                .unwrap_or(0.0);
            set_anim_time.set(time);
        }) as Box<dyn FnMut()>);

        let _ = web_sys::window()
            .unwrap()
            .set_interval_with_callback_and_timeout_and_arguments_0(
                interval_cb.as_ref().unchecked_ref(),
                33,
            );
        interval_cb.forget();
    });

    // Render canvas each frame
    Effect::new(move |_| {
        let time = anim_time.get();
        let current_orbs = orbs.get();
        let drag = dragging_id.get();

        if let Some(canvas) = get_canvas_el() {
            if let Ok(Some(ctx_obj)) = canvas.get_context("2d") {
                if let Ok(ctx) = ctx_obj.dyn_into::<CanvasRenderingContext2d>() {
                    draw_terrain(
                        &ctx,
                        canvas.width() as f64,
                        canvas.height() as f64,
                        &current_orbs,
                        time,
                        drag,
                    );
                }
            }
        }
    });

    // Window resize handler
    Effect::new(move |_| {
        let cb = Closure::wrap(Box::new(move || {
            resize_canvas();
        }) as Box<dyn FnMut()>);
        if let Some(window) = web_sys::window() {
            window.set_onresize(Some(cb.as_ref().unchecked_ref()));
        }
        cb.forget();
    });

    // Keyboard shortcuts
    Effect::new(move |_| {
        let cb = Closure::wrap(Box::new(move |ev: web_sys::KeyboardEvent| {
            if ev.key() == " " {
                ev.prevent_default();
                ensure_engine();
                engine.with_value(|cell| {
                    if let Some(ref mut eng) = *cell.borrow_mut() {
                        let _ = eng.toggle();
                        set_is_playing.set(eng.is_playing());
                    }
                });
            }
        }) as Box<dyn FnMut(web_sys::KeyboardEvent)>);
        if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
            doc.set_onkeydown(Some(cb.as_ref().unchecked_ref()));
        }
        cb.forget();
    });

    // Scroll = master volume
    Effect::new(move |_| {
        let cb = Closure::wrap(Box::new(move |ev: web_sys::WheelEvent| {
            let delta = ev.delta_y();
            let change: f32 = if delta > 0.0 { -0.02 } else { 0.02 };
            let new_vol = (master_vol.get_untracked() + change).clamp(0.0, 1.0);
            set_master_vol.set(new_vol);
            engine.with_value(|cell| {
                if let Some(ref eng) = *cell.borrow_mut() {
                    eng.set_master_volume(new_vol);
                }
            });
            save_session();
        }) as Box<dyn FnMut(web_sys::WheelEvent)>);
        if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
            doc.set_onwheel(Some(cb.as_ref().unchecked_ref()));
        }
        cb.forget();
    });

    let add_source = move |kind: SourceKind, x: f32, y: f32| -> usize {
        ensure_engine();
        let mut id = 0;
        next_id.update_value(|v| {
            id = *v;
            *v += 1;
        });

        engine.with_value(|cell| {
            if let Some(ref mut eng) = *cell.borrow_mut() {
                let canvas_w = get_canvas_el().map(|c| c.width() as f32).unwrap_or(800.0);
                let canvas_h = get_canvas_el().map(|c| c.height() as f32).unwrap_or(500.0);

                let px = x * canvas_w;
                let py = (1.0 - y) * canvas_h;

                if let Ok(source) =
                    SoundSource::new(eng.context(), kind, eng.master_gain_node(), x, y)
                {
                    // For noise sources, set initial filter cutoff from x position
                    if kind.is_noise() {
                        let cutoff = x_to_cutoff(x);
                        eng.add_source(source);
                        if let Some(s) = eng.sources_mut().last_mut() {
                            s.filter_chain_mut()
                                .set_filter(FilterKind::Lowpass, cutoff, 1.0);
                        }
                    } else {
                        eng.add_source(source);
                    }

                    set_orbs.update(|o| {
                        o.push(OrbData::new(id, kind, px as f64, py as f64, x, y));
                    });
                }
            }
        });
        id
    };

    let remove_source_by_id = move |id: usize| {
        engine.with_value(|cell| {
            if let Some(ref mut eng) = *cell.borrow_mut() {
                let current_orbs = orbs.get_untracked();
                if let Some(idx) = current_orbs.iter().position(|o| o.id == id) {
                    eng.remove_source(idx);
                    set_orbs.update(|o| {
                        o.remove(idx);
                    });
                    if hovered_orb_id.get_untracked() == Some(id) {
                        set_hovered_orb_id.set(None);
                    }
                }
            }
        });
        save_session();
    };

    let toggle_orb_binaural = move |id: usize| {
        engine.with_value(|cell| {
            if let Some(ref mut eng) = *cell.borrow_mut() {
                let ctx = eng.context().clone();
                let master = eng.master_gain_node().clone();

                let current_orbs = orbs.get_untracked();
                if let Some(idx) = current_orbs.iter().position(|o| o.id == id) {
                    if let Some(source) = eng.sources_mut().get_mut(idx) {
                        if let Ok(is_active) = source.toggle_binaural(&ctx, &master, 6.0) {
                            set_orbs.update(|o| {
                                if let Some(orb) = o.get_mut(idx) {
                                    orb.binaural_active = is_active;
                                }
                            });
                        }
                    }
                }
            }
        });
        save_session();
    };

    let toggle_orb_filter = move |id: usize, filter_kind: FilterKind| {
        engine.with_value(|cell| {
            if let Some(ref mut eng) = *cell.borrow_mut() {
                let current_orbs = orbs.get_untracked();
                if let Some(idx) = current_orbs.iter().position(|o| o.id == id) {
                    let orb = &current_orbs[idx];
                    let already_active = orb.active_filters.contains(&filter_kind);

                    if let Some(source) = eng.sources_mut().get_mut(idx) {
                        if already_active {
                            // Remove this filter
                            source.filter_chain_mut().remove_filter_by_kind(filter_kind);
                            set_orbs.update(|o| {
                                if let Some(orb) = o.get_mut(idx) {
                                    orb.active_filters.retain(|&f| f != filter_kind);
                                }
                            });
                        } else {
                            // Add this filter — use position-based freq for noise sources
                            let (freq, q) = if orb.kind.is_noise() {
                                match filter_kind {
                                    FilterKind::Lowpass | FilterKind::Bandpass => {
                                        (x_to_cutoff(orb.norm_x), 1.0)
                                    }
                                    FilterKind::Highpass => (x_to_cutoff(orb.norm_x), 1.0),
                                    FilterKind::Notch => (x_to_cutoff(orb.norm_x), 5.0),
                                }
                            } else {
                                match filter_kind {
                                    FilterKind::Lowpass => (1000.0, 1.0),
                                    FilterKind::Highpass => (200.0, 1.0),
                                    FilterKind::Bandpass => (500.0, 2.0),
                                    FilterKind::Notch => (500.0, 5.0),
                                }
                            };
                            source.filter_chain_mut().add_filter(filter_kind, freq, q);
                            set_orbs.update(|o| {
                                if let Some(orb) = o.get_mut(idx) {
                                    orb.active_filters.push(filter_kind);
                                }
                            });
                        }
                    }
                }
            }
        });
        save_session();
    };

    let clear_all_sources = move || {
        engine.with_value(|cell| {
            if let Some(ref mut eng) = *cell.borrow_mut() {
                let sources = eng.sources_mut();
                for s in sources.iter() {
                    s.disconnect();
                }
                sources.clear();
            }
        });
        binaural_store.with_value(|cell| {
            if let Some(ref b) = *cell.borrow() {
                b.disconnect();
            }
            *cell.borrow_mut() = None;
        });
        set_orbs.set(Vec::new());
        next_id.set_value(0);
        set_binaural_active.set(false);
        set_hovered_orb_id.set(None);
    };

    let load_preset = move |preset: &Preset| {
        clear_all_sources();
        ensure_engine();

        set_master_vol.set(preset.master_volume);
        engine.with_value(|cell| {
            if let Some(ref eng) = *cell.borrow_mut() {
                eng.set_master_volume(preset.master_volume);
            }
        });

        for ps in &preset.sources {
            let id = add_source(ps.kind, ps.x, ps.y);
            engine.with_value(|cell| {
                if let Some(ref mut eng) = *cell.borrow_mut() {
                    if let Some(source) = eng.sources_mut().last_mut() {
                        source.set_gain(ps.gain);
                        if let (Some(fk), Some(freq), Some(q)) =
                            (ps.filter_kind, ps.filter_freq, ps.filter_q)
                        {
                            source.filter_chain_mut().set_filter(fk, freq, q);
                        }
                    }
                }
            });
            set_orbs.update(|o| {
                if let Some(orb) = o.iter_mut().find(|o| o.id == id) {
                    orb.gain = ps.gain;
                    orb.update_radius();
                    if let Some(fk) = ps.filter_kind {
                        if !orb.active_filters.contains(&fk) {
                            orb.active_filters.push(fk);
                        }
                    }
                }
            });
        }

        if let Some(ref bdata) = preset.binaural {
            engine.with_value(|cell| {
                if let Some(ref eng) = *cell.borrow_mut() {
                    if let Ok(b) = BinauralBeat::new(
                        eng.context(),
                        eng.master_gain_node(),
                        bdata.base_freq,
                        bdata.beat_freq,
                    ) {
                        binaural_store.with_value(|bc| {
                            *bc.borrow_mut() = Some(b);
                        });
                        set_binaural_active.set(true);
                    }
                }
            });
        }
    };

    // Load saved session on startup
    let load_preset_clone = load_preset.clone();
    Effect::new(move |_| {
        if let Some(session) = persistence::load_session() {
            load_preset_clone(&session);
        }
    });

    let presets_for_dock = built_in_presets();
    let presets_for_loading = built_in_presets();

    let on_action = Callback::new(move |action: DockAction| match action {
        DockAction::TogglePlay => {
            ensure_engine();
            engine.with_value(|cell| {
                if let Some(ref mut eng) = *cell.borrow_mut() {
                    let _ = eng.toggle();
                    set_is_playing.set(eng.is_playing());
                }
            });
        }
        DockAction::AddSource(kind) => {
            let (x, y) = default_position(kind);
            let _ = add_source(kind, x, y);
            set_active_preset.set(None);
            save_session();
        }
        DockAction::RemoveLast => {
            engine.with_value(|cell| {
                if let Some(ref mut eng) = *cell.borrow_mut() {
                    let len = eng.sources().len();
                    if len > 0 {
                        eng.remove_source(len - 1);
                        set_orbs.update(|o| {
                            o.pop();
                        });
                    }
                }
            });
            set_active_preset.set(None);
            save_session();
        }
        DockAction::LoadPreset(idx) => {
            if let Some(preset) = presets_for_loading.get(idx) {
                load_preset(preset);
                set_active_preset.set(Some(idx));
                save_session();
            }
        }
        DockAction::ToggleBinaural => {
            ensure_engine();
            let active = binaural_active.get_untracked();
            if active {
                binaural_store.with_value(|cell| {
                    if let Some(ref b) = *cell.borrow() {
                        b.disconnect();
                    }
                    *cell.borrow_mut() = None;
                });
                set_binaural_active.set(false);
            } else {
                engine.with_value(|cell| {
                    if let Some(ref eng) = *cell.borrow_mut() {
                        if let Ok(b) =
                            BinauralBeat::new(eng.context(), eng.master_gain_node(), 200.0, 6.0)
                        {
                            binaural_store.with_value(|bc| {
                                *bc.borrow_mut() = Some(b);
                            });
                            set_binaural_active.set(true);
                        }
                    }
                });
            }
            save_session();
        }
    });

    let on_volume = Callback::new(move |vol: f32| {
        set_master_vol.set(vol);
        engine.with_value(|cell| {
            if let Some(ref eng) = *cell.borrow_mut() {
                eng.set_master_volume(vol);
            }
        });
        save_session();
    });

    // Mouse handlers for dragging orbs
    let on_pointer_down = move |ev: MouseEvent| {
        if let Some(canvas) = get_canvas_el() {
            let rect = canvas.get_bounding_client_rect();
            let px = ev.client_x() as f64 - rect.left();
            let py = ev.client_y() as f64 - rect.top();

            let current_orbs = orbs.get_untracked();
            for orb in current_orbs.iter().rev() {
                if orb.contains(px, py) {
                    set_dragging_id.set(Some(orb.id));
                    set_hovered_orb_id.set(None);
                    set_is_hovering_menu.set(false);
                    return;
                }
            }
            // Clicked on empty canvas — dismiss any open menu
            set_hovered_orb_id.set(None);
            set_is_hovering_menu.set(false);
        }
    };

    let on_pointer_move = move |ev: MouseEvent| {
        if let Some(canvas) = get_canvas_el() {
            let rect = canvas.get_bounding_client_rect();
            let px = ev.client_x() as f64 - rect.left();
            let py = ev.client_y() as f64 - rect.top();
            let w = canvas.width() as f64;
            let h = canvas.height() as f64;

            if let Some(drag_id) = dragging_id.get_untracked() {
                let norm_x = (px / w) as f32;
                let norm_y = (1.0 - py / h) as f32;

                set_orbs.update(|orbs_vec| {
                    if let Some(orb) = orbs_vec.iter_mut().find(|o| o.id == drag_id) {
                        orb.x = px.clamp(0.0, w);
                        orb.y = py.clamp(0.0, h);
                        orb.norm_x = norm_x.clamp(0.0, 1.0);
                        orb.norm_y = norm_y.clamp(0.0, 1.0);
                        orb.gain = norm_y.clamp(0.0, 1.0);
                        if !orb.kind.is_noise() {
                            orb.freq = norm_x * 1000.0;
                        }
                        orb.update_radius();
                    }
                });

                engine.with_value(|cell| {
                    if let Some(ref mut eng) = *cell.borrow_mut() {
                        let current_orbs = orbs.get_untracked();
                        if let Some(idx) = current_orbs.iter().position(|o| o.id == drag_id) {
                            if let Some(source) = eng.sources_mut().get_mut(idx) {
                                source.set_gain(norm_y.clamp(0.0, 1.0));
                                if current_orbs[idx].kind.is_noise() {
                                    // For noise: x-axis controls filter cutoff
                                    let cutoff = x_to_cutoff(norm_x.clamp(0.0, 1.0));
                                    source.set_filter_cutoff(cutoff);
                                } else {
                                    // For oscillators: x-axis controls frequency
                                    source.set_frequency(norm_x * 1000.0);
                                }
                            }
                        }
                    }
                });

                set_active_preset.set(None);
            } else {
                // Hover logic
                let current_orbs = orbs.get_untracked();
                let mut found = None;
                for orb in current_orbs.iter().rev() {
                    if orb.contains(px, py) {
                        found = Some(orb.id);
                        break;
                    }
                }

                if found.is_some() {
                    set_hovered_orb_id.set(found);
                } else if !is_hovering_menu.get_untracked() {
                    // Keep hover alive if mouse is in approach zone above the hovered orb
                    // (bridging the gap between orb and menu buttons)
                    let in_approach = hovered_orb_id
                        .get_untracked()
                        .and_then(|hid| current_orbs.iter().find(|o| o.id == hid))
                        .is_some_and(|orb| {
                            let dx = (px - orb.x).abs();
                            let dy = orb.y - py; // positive = above orb
                            dx < 100.0 && dy > 0.0 && dy < orb.radius as f64 + 100.0
                        });
                    if !in_approach {
                        set_hovered_orb_id.set(None);
                    }
                }
            }
        }
    };

    let on_pointer_up = move |_ev: MouseEvent| {
        if dragging_id.get_untracked().is_some() {
            set_dragging_id.set(None);
            save_session();
        }
    };

    // Touch handlers
    let on_touch_start = move |ev: TouchEvent| {
        ev.prevent_default();
        if let Some(touch) = ev.touches().item(0) {
            if let Some(canvas) = get_canvas_el() {
                let rect = canvas.get_bounding_client_rect();
                let px = touch.client_x() as f64 - rect.left();
                let py = touch.client_y() as f64 - rect.top();

                let current_orbs = orbs.get_untracked();
                for orb in current_orbs.iter().rev() {
                    if orb.contains(px, py) {
                        set_dragging_id.set(Some(orb.id));
                        set_hovered_orb_id.set(None);
                        return;
                    }
                }
            }
        }
    };

    let on_touch_move = move |ev: TouchEvent| {
        ev.prevent_default();
        if let Some(drag_id) = dragging_id.get_untracked() {
            if let Some(touch) = ev.touches().item(0) {
                if let Some(canvas) = get_canvas_el() {
                    let rect = canvas.get_bounding_client_rect();
                    let px = touch.client_x() as f64 - rect.left();
                    let py = touch.client_y() as f64 - rect.top();
                    let w = canvas.width() as f64;
                    let h = canvas.height() as f64;

                    let norm_x = (px / w) as f32;
                    let norm_y = (1.0 - py / h) as f32;

                    set_orbs.update(|orbs_vec| {
                        if let Some(orb) = orbs_vec.iter_mut().find(|o| o.id == drag_id) {
                            orb.x = px.clamp(0.0, w);
                            orb.y = py.clamp(0.0, h);
                            orb.norm_x = norm_x.clamp(0.0, 1.0);
                            orb.norm_y = norm_y.clamp(0.0, 1.0);
                            orb.gain = norm_y.clamp(0.0, 1.0);
                            if !orb.kind.is_noise() {
                                orb.freq = norm_x * 1000.0;
                            }
                            orb.update_radius();
                        }
                    });

                    engine.with_value(|cell| {
                        if let Some(ref mut eng) = *cell.borrow_mut() {
                            let current_orbs = orbs.get_untracked();
                            if let Some(idx) = current_orbs.iter().position(|o| o.id == drag_id) {
                                if let Some(source) = eng.sources_mut().get_mut(idx) {
                                    source.set_gain(norm_y.clamp(0.0, 1.0));
                                    if current_orbs[idx].kind.is_noise() {
                                        let cutoff = x_to_cutoff(norm_x.clamp(0.0, 1.0));
                                        source.set_filter_cutoff(cutoff);
                                    } else {
                                        source.set_frequency(norm_x * 1000.0);
                                    }
                                }
                            }
                        }
                    });
                }
            }
        }
    };

    let on_touch_end = move |_ev: TouchEvent| {
        if dragging_id.get_untracked().is_some() {
            set_dragging_id.set(None);
            save_session();
        }
    };

    // Filter kinds for the hover menu
    let filter_kinds: [(FilterKind, &str); 4] = [
        (FilterKind::Lowpass, "LP"),
        (FilterKind::Highpass, "HP"),
        (FilterKind::Bandpass, "BP"),
        (FilterKind::Notch, "Notch"),
    ];

    view! {
        <div class="fixed inset-0 w-full h-dvh">
            <canvas
                node_ref=canvas_ref
                class="absolute inset-0 w-full h-dvh block cursor-crosshair touch-none"
                on:mousedown=on_pointer_down
                on:mousemove=on_pointer_move
                on:mouseup=on_pointer_up
                on:mouseleave=on_pointer_up
                on:touchstart=on_touch_start
                on:touchmove=on_touch_move
                on:touchend=on_touch_end
            />

            // Orb Hover Menu
            {move || {
                hovered_orb_id.get().and_then(|id| {
                    orbs.get().iter().find(|o| o.id == id).map(|orb| {
                        let menu_top = orb.y - orb.radius - 15.0;
                        let left = orb.x;
                        let is_binaural = orb.binaural_active;
                        let current_filters = orb.active_filters.clone();

                        view! {
                            <div
                                class="animate-fade-in absolute z-50 pointer-events-auto"
                                style:top=format!("{}px", menu_top)
                                style:left=format!("{}px", left)
                                style:transform="translate(-50%, -100%)"
                                on:mouseenter=move |_| set_is_hovering_menu.set(true)
                                on:mouseleave=move |_| {
                                    set_is_hovering_menu.set(false);
                                }
                                on:mousedown=move |ev: MouseEvent| {
                                    ev.stop_propagation();
                                }
                            >
                                <div class="flex flex-col items-center gap-1">
                                    // Filter buttons row
                                    <div class="flex gap-1">
                                        {filter_kinds
                                            .iter()
                                            .map(|(fk, label)| {
                                                let fk = *fk;
                                                let is_active = current_filters.contains(&fk);
                                                let border_color = fk.color();
                                                view! {
                                                    <button
                                                        class="rounded-[10px] tracking-[0.5px] font-medium text-[9px] px-1.5 py-[3px] cursor-pointer backdrop-blur-sm font-mono whitespace-nowrap transition-all duration-200"
                                                        style:background=move || {
                                                            if is_active {
                                                                format!("{}22", border_color)
                                                            } else {
                                                                "rgba(0,0,0,0.8)".into()
                                                            }
                                                        }
                                                        style:border=move || {
                                                            if is_active {
                                                                format!("1px solid {}", border_color)
                                                            } else {
                                                                "1px solid rgba(255,255,255,0.2)".into()
                                                            }
                                                        }
                                                        style:color=move || {
                                                            if is_active {
                                                                border_color.to_string()
                                                            } else {
                                                                "rgba(255,255,255,0.6)".into()
                                                            }
                                                        }
                                                        on:click=move |_| toggle_orb_filter(id, fk)
                                                    >
                                                        {*label}
                                                    </button>
                                                }
                                            })
                                            .collect::<Vec<_>>()}
                                    </div>
                                    // Action buttons row
                                    <div class="flex gap-1">
                                        <button
                                            class="text-[10px] px-2 py-1 bg-black/80 backdrop-blur-sm rounded-2xl font-mono cursor-pointer whitespace-nowrap transition-all duration-200"
                                            style:border=move || if is_binaural { "1px solid #00CED1" } else { "1px solid rgba(255,255,255,0.2)" }
                                            style:color=move || if is_binaural { "#00CED1" } else { "#b0b0b0" }
                                            on:click=move |_| toggle_orb_binaural(id)
                                        >
                                            "Binaural"
                                        </button>
                                        <button
                                            class="text-[10px] px-2 py-1 bg-black/80 backdrop-blur-sm border border-[rgba(255,80,80,0.25)] text-[rgba(255,80,80,0.6)] rounded-2xl font-mono cursor-pointer whitespace-nowrap transition-all duration-200 hover:border-[rgba(255,80,80,0.5)] hover:text-[#ff5050]"
                                            on:click=move |_| remove_source_by_id(id)
                                        >
                                            "Delete"
                                        </button>
                                    </div>
                                </div>
                            </div>
                        }
                    })
                })
            }}

            <Dock
                presets=presets_for_dock
                is_playing=is_playing
                on_action=on_action
                binaural_active=binaural_active
                master_volume=master_vol
                on_volume=on_volume
                active_preset=active_preset
            />
        </div>
    }
}
