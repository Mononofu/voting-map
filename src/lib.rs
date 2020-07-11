mod utils;

use wasm_bindgen::prelude::*;
use wasm_bindgen::Clamped;
use web_sys::{CanvasRenderingContext2d, ImageData};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
// TODO(swj): Comment out for deployment, increases the binary size quite a lot.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

#[wasm_bindgen]
pub fn render(width: u32, height: u32) -> Result<Vec<u8>, JsValue> {
    let mut data = Vec::new();

    for x in 0..width {
        for y in 0..height {
            data.push(0);
            data.push(200);
            data.push(100);
            data.push(255);
        }
    }

    Ok(data)
}
