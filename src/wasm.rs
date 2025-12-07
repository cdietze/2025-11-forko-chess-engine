use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello from forko");
}

#[wasm_bindgen]
pub fn version() -> String {
    crate::util::engine_display_name()
}
