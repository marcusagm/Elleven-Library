use std::collections::{HashSet, VecDeque};
use std::sync::Mutex;

pub struct ThumbnailPriorityState {
    pub priority_ids: Mutex<VecDeque<i64>>,
}

impl Default for ThumbnailPriorityState {
    fn default() -> Self {
        Self {
            priority_ids: Mutex::new(VecDeque::new()),
        }
    }
}

impl ThumbnailPriorityState {
    /// Overwrites the current priority queue with the new IDs, keeping them
    /// at the front so they are processed First (LIFO per block).
    pub fn set_priority(&self, ids: Vec<i64>) {
        if dbg!(ids.is_empty()) {
            return;
        }

        if let Ok(mut deque) = self.priority_ids.lock() {
            deque.clear();
            // Since we want LIFO at the UI level (what the user scrolled into view most recently),
            // and the UI sends them dynamically, we essentially just prioritize the latest list.
            for id in ids {
                // Front receives newest first, to be popped front later
                deque.push_front(id);
            }
        }
    }

    /// Safely extracts up to `max_size` unique IDs from the priority queue.
    pub fn take_batch(&self, max_size: usize) -> Vec<i64> {
        let mut batch = Vec::new();
        if let Ok(mut deque) = self.priority_ids.lock() {
            let mut seen = HashSet::new();
            while let Some(id) = deque.pop_front() {
                if !seen.contains(&id) {
                    seen.insert(id);
                    batch.push(id);
                }
                if batch.len() >= max_size {
                    break;
                }
            }
        }
        batch
    }
}
