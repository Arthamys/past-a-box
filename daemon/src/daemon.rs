use std::sync::{Arc, Mutex};
use api::common::clipping::Clipping;

/// The Daemon struct represents the running instance of the Past-a-Box daemon.
/// Currently, the daemon only holds the clippings as an in memory vector.
pub struct Daemon {
    pub storage: Arc<Mutex<Vec<Clipping>>>,
}

impl Daemon {
    /// Create a new Daemon instance
    pub fn new() -> Daemon {
        Daemon {
            storage: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Add a new clipping to the daemon's clipping storage
    pub fn add_clipping(&self, clip: Clipping) {
        let mut clippings = self.storage.lock().expect("Could not lock storage mutex");
        clippings.push(clip);
    }
}
