use std::sync::atomic::{AtomicUsize, Ordering};

pub fn new_node_id() -> usize {
    static COUNTER: AtomicUsize = AtomicUsize::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}
