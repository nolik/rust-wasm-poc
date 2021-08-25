use serde_derive::{Deserialize, Serialize};
use strum_macros::{EnumIter, ToString};

#[derive(Debug, Serialize, Deserialize)]
pub struct State {
    pub entries: Vec<Entry>,
    pub value: String
}

impl State {
    pub fn toggle(&mut self, idx: usize) {
        let entry = self
            .entries
            .iter_mut()
            .nth(idx)
            .unwrap();
        entry.completed = !entry.completed;

        // copy to clipboard
        let window: web_sys::Window = web_sys::window().expect("window not available");
        let navigator: web_sys::Navigator = window.navigator();
        let clip: web_sys::Clipboard = navigator.clipboard().expect("Clipboard not available");
        let promise  = clip.write_text(&entry.description);
        wasm_bindgen_futures::spawn_local(async {
            wasm_bindgen_futures::JsFuture::from(promise).await;
        });
    }

    pub fn remove(&mut self, idx: usize) {
        let idx = {
            let entries = self
                .entries
                .iter()
                .enumerate()
                .collect::<Vec<_>>();
            let &(idx, _) = entries.get(idx).unwrap();
            idx
        };
        self.entries.remove(idx);
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Entry {
    pub description: String,
    pub completed: bool
}
