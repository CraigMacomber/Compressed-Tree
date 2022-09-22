#![cfg(target_arch = "wasm32")]

#[wasm_bindgen_test]
fn minimal() {
    drop(Box::new(1));
}
