use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/src/browser.js")]
extern "C" {
    #[wasm_bindgen(js_name = onGridUpdate)]
    fn on_grid_update();
}

pub(crate) struct Browser;

impl Browser {
    pub(crate) fn trigger_grid_update() {
        on_grid_update();
    }
}
