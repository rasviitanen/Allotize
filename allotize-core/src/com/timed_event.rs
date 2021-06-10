use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn setInterval(closure: &Closure<dyn FnMut()>, millis: u32) -> f64;
    fn clearInterval(token: f64);
}

#[wasm_bindgen]
pub struct TimedEvent {
    // Closure will be uncallable if we don't save it
    _closure: Closure<dyn FnMut()>,
    token: f64,
}

impl TimedEvent {
    pub fn new(closure: Closure<dyn FnMut()>, millis: u32) -> TimedEvent {
        // Pass the closuer to JS, to run every n milliseconds.
        let token = setInterval(&closure, millis);

        TimedEvent {
            _closure: closure,
            token,
        }
    }
}

// When the Interval is destroyed, cancel its `setInterval` timer.
impl Drop for TimedEvent {
    fn drop(&mut self) {
        clearInterval(self.token);
    }
}
