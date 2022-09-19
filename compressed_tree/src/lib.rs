// #![no_std]

use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use web_sys::console;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn greet(name: &str) {
    console::log_1(&format!("Hello, {}!", name).into());
}


#[cfg(test)]
mod tests {
    #[test]
    fn basic_test() {
        assert_eq!(0, 0);
    }
}
