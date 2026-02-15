use wasm_bindgen::prelude::*;

use crate::ui::orb::OrbData;

#[wasm_bindgen(module = "/js/three_scene.js")]
extern "C" {
    #[wasm_bindgen(js_name = "initScene")]
    pub fn init_scene(canvas: &web_sys::HtmlCanvasElement);

    #[wasm_bindgen(js_name = "addOrb")]
    pub fn add_orb(id: usize, color_hex: &str, norm_x: f32, norm_y: f32, radius: f64);

    #[wasm_bindgen(js_name = "updateOrb")]
    pub fn update_orb(
        id: usize,
        norm_x: f32,
        norm_y: f32,
        radius: f64,
        gain: f32,
        freq: f32,
        binaural: bool,
        filter_colors: &js_sys::Array,
    );

    #[wasm_bindgen(js_name = "removeOrb")]
    pub fn remove_orb(id: usize);

    #[wasm_bindgen(js_name = "removeAllOrbs")]
    pub fn remove_all_orbs();

    #[wasm_bindgen(js_name = "render")]
    pub fn render(time: f64);

    #[wasm_bindgen(js_name = "resize")]
    pub fn resize(w: f64, h: f64);

    #[wasm_bindgen(js_name = "getOrbAtPoint")]
    pub fn get_orb_at_point(px: f64, py: f64, w: f64, h: f64) -> i32;

    #[wasm_bindgen(js_name = "getOrbScreenPos")]
    fn get_orb_screen_pos_js(id: usize, w: f64, h: f64) -> JsValue;
}

/// Sync an OrbData to the Three.js scene
pub fn sync_orb(orb: &OrbData) {
    let filter_colors = js_sys::Array::new();
    for fk in &orb.active_filters {
        filter_colors.push(&JsValue::from_str(fk.color()));
    }

    update_orb(
        orb.id,
        orb.norm_x,
        orb.norm_y,
        orb.radius,
        orb.gain,
        orb.freq,
        orb.binaural_active,
        &filter_colors,
    );
}

/// Get the screen position and radius of an orb.
/// Returns (x, y, radius) in screen pixels, or None if not found.
pub fn get_screen_pos(id: usize, w: f64, h: f64) -> Option<(f64, f64, f64)> {
    let val = get_orb_screen_pos_js(id, w, h);
    if val.is_null() || val.is_undefined() {
        return None;
    }

    let x = js_sys::Reflect::get(&val, &JsValue::from_str("x"))
        .ok()?
        .as_f64()?;
    let y = js_sys::Reflect::get(&val, &JsValue::from_str("y"))
        .ok()?
        .as_f64()?;
    let radius = js_sys::Reflect::get(&val, &JsValue::from_str("radius"))
        .ok()?
        .as_f64()?;

    Some((x, y, radius))
}
