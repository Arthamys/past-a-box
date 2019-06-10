use api::common::clipping::Clipping;
use std::sync::{Arc, Mutex};

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
}
