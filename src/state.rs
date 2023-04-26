use serde_derive::{Deserialize, Serialize};

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

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Entry {
    pub description: String,
    pub completed: bool
}
