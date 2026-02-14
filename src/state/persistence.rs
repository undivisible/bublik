use crate::state::Preset;
use wasm_bindgen::prelude::*;

const STORAGE_KEY: &str = "bublik_presets";

pub fn save_presets(presets: &[Preset]) -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("no window")?;
    let storage = window
        .local_storage()?
        .ok_or("no localStorage")?;
    let json = serde_json::to_string(presets).map_err(|e| JsValue::from_str(&e.to_string()))?;
    storage.set_item(STORAGE_KEY, &json)?;
    Ok(())
}

pub fn load_presets() -> Vec<Preset> {
    let window = match web_sys::window() {
        Some(w) => w,
        None => return Vec::new(),
    };
    let storage = match window.local_storage() {
        Ok(Some(s)) => s,
        _ => return Vec::new(),
    };
    let json = match storage.get_item(STORAGE_KEY) {
        Ok(Some(s)) => s,
        _ => return Vec::new(),
    };
    serde_json::from_str(&json).unwrap_or_default()
}

pub fn encode_url_state(preset: &Preset) -> String {
    let json = serde_json::to_string(preset).unwrap_or_default();
    let encoded = js_sys::encode_uri_component(&json);
    format!("#{}", encoded)
}

pub fn decode_url_state() -> Option<Preset> {
    let window = web_sys::window()?;
    let hash = window.location().hash().ok()?;
    if hash.len() <= 1 {
        return None;
    }
    let encoded = &hash[1..];
    let decoded = js_sys::decode_uri_component(encoded).ok()?;
    serde_json::from_str(&String::from(decoded)).ok()
}
