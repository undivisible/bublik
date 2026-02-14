use leptos::prelude::*;
use leptos::reactive::owner::LocalStorage;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, MouseEvent, TouchEvent};

use crate::audio::binaural::BinauralBeat;
use crate::audio::context::AudioEngine;
use crate::audio::sources::{SoundSource, SourceKind};
use crate::state::presets::{built_in_presets, Preset};
use crate::ui::dock::{Dock, DockAction};
use crate::ui::orb::OrbData;
use crate::ui::terrain::draw_terrain;

#[component]
pub fn App() -> impl IntoView {
    let (is_playing, set_is_playing) = signal(false);
    let (master_vol, set_master_vol) = signal(0.7f32);
    let (binaural_active, set_binaural_active) = signal(false);
    let (orbs, set_orbs) = signal(Vec::<OrbData>::new());
    let (dragging_id, set_dragging_id) = signal(Option::<usize>::None);
    let (next_id, set_next_id) = signal(0usize);
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

    let resize_canvas = move || {
        if let Some(canvas) = get_canvas_el() {
            let window = web_sys::window().unwrap();
            let w = window.inner_width().unwrap().as_f64().unwrap();
            let h = window.inner_height().unwrap().as_f64().unwrap() - 100.0;
            canvas.set_width(w as u32);
            canvas.set_height(h.max(200.0) as u32);
        }
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
        }) as Box<dyn FnMut(web_sys::WheelEvent)>);
        if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
            doc.set_onwheel(Some(cb.as_ref().unchecked_ref()));
        }
        cb.forget();
    });

    let add_source = move |kind: SourceKind, x: f32, y: f32| {
        ensure_engine();
        let id = next_id.get_untracked();
        set_next_id.set(id + 1);

        engine.with_value(|cell| {
            if let Some(ref mut eng) = *cell.borrow_mut() {
                let canvas_w = get_canvas_el().map(|c| c.width() as f32).unwrap_or(800.0);
                let canvas_h = get_canvas_el().map(|c| c.height() as f32).unwrap_or(500.0);

                let px = x * canvas_w;
                let py = (1.0 - y) * canvas_h;

                if let Ok(source) =
                    SoundSource::new(eng.context(), kind, eng.master_gain_node(), x, y)
                {
                    eng.add_source(source);
                    set_orbs.update(|o| {
                        o.push(OrbData::new(id, kind, px as f64, py as f64, y));
                    });
                }
            }
        });
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
        set_next_id.set(0);
        set_binaural_active.set(false);
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
            add_source(ps.kind, ps.x, ps.y);
            if ps.filter_kind.is_some() {
                engine.with_value(|cell| {
                    if let Some(ref mut eng) = *cell.borrow_mut() {
                        if let Some(source) = eng.sources_mut().last_mut() {
                            source.set_gain(ps.gain);
                            if let (Some(freq), Some(q)) = (ps.filter_freq, ps.filter_q) {
                                source.filter_chain_mut().set_lowpass(freq, q);
                            }
                        }
                    }
                });
            }
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
            let x = 0.2 + (js_sys::Math::random() as f32) * 0.6;
            let y = 0.3 + (js_sys::Math::random() as f32) * 0.4;
            add_source(kind, x, y);
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
        }
        DockAction::LoadPreset(idx) => {
            if let Some(preset) = presets_for_loading.get(idx) {
                load_preset(preset);
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
        }
    });

    let on_volume = Callback::new(move |vol: f32| {
        set_master_vol.set(vol);
        engine.with_value(|cell| {
            if let Some(ref eng) = *cell.borrow_mut() {
                eng.set_master_volume(vol);
            }
        });
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
                    return;
                }
            }
        }
    };

    let on_pointer_move = move |ev: MouseEvent| {
        if let Some(drag_id) = dragging_id.get_untracked() {
            if let Some(canvas) = get_canvas_el() {
                let rect = canvas.get_bounding_client_rect();
                let px = ev.client_x() as f64 - rect.left();
                let py = ev.client_y() as f64 - rect.top();
                let w = canvas.width() as f64;
                let h = canvas.height() as f64;

                let norm_x = (px / w) as f32;
                let norm_y = (1.0 - py / h) as f32;

                set_orbs.update(|orbs_vec| {
                    if let Some(orb) = orbs_vec.iter_mut().find(|o| o.id == drag_id) {
                        orb.x = px.clamp(0.0, w);
                        orb.y = py.clamp(0.0, h);
                        orb.gain = norm_y.clamp(0.0, 1.0);
                        orb.freq = norm_x * 1000.0;
                        orb.update_radius();
                    }
                });

                engine.with_value(|cell| {
                    if let Some(ref mut eng) = *cell.borrow_mut() {
                        let current_orbs = orbs.get_untracked();
                        if let Some(idx) = current_orbs.iter().position(|o| o.id == drag_id) {
                            if let Some(source) = eng.sources_mut().get_mut(idx) {
                                source.set_gain(norm_y.clamp(0.0, 1.0));
                                source.set_frequency(norm_x * 1000.0);
                            }
                        }
                    }
                });
            }
        }
    };

    let on_pointer_up = move |_ev: MouseEvent| {
        set_dragging_id.set(None);
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
                            orb.gain = norm_y.clamp(0.0, 1.0);
                            orb.freq = norm_x * 1000.0;
                            orb.update_radius();
                        }
                    });

                    engine.with_value(|cell| {
                        if let Some(ref mut eng) = *cell.borrow_mut() {
                            let current_orbs = orbs.get_untracked();
                            if let Some(idx) =
                                current_orbs.iter().position(|o| o.id == drag_id)
                            {
                                if let Some(source) = eng.sources_mut().get_mut(idx) {
                                    source.set_gain(norm_y.clamp(0.0, 1.0));
                                    source.set_frequency(norm_x * 1000.0);
                                }
                            }
                        }
                    });
                }
            }
        }
    };

    let on_touch_end = move |_ev: TouchEvent| {
        set_dragging_id.set(None);
    };

    view! {
        <div class="app">
            <canvas
                node_ref=canvas_ref
                class="terrain-canvas"
                on:mousedown=on_pointer_down
                on:mousemove=on_pointer_move
                on:mouseup=on_pointer_up
                on:mouseleave=on_pointer_up
                on:touchstart=on_touch_start
                on:touchmove=on_touch_move
                on:touchend=on_touch_end
            />
            <Dock
                presets=presets_for_dock
                is_playing=is_playing
                on_action=on_action
                binaural_active=binaural_active
                master_volume=master_vol
                on_volume=on_volume
            />
        </div>
    }
}
