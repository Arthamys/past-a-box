use api::common::clipping::Clipping;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

/// Stores the clippings that have accumulated during the session
#[derive(Debug)]
pub struct ClippingStorage {
    store: Arc<Mutex<Vec<Clipping>>>,
    tx: Sender<Clipping>,
}

impl ClippingStorage {
    /// Create a new storage that will live as long as the program.
    pub fn new() -> (Self, JoinHandle<()>) {
        let (tx, rx) = channel();
        let store = Arc::new(Mutex::new(Vec::new()));
        let store_clone = Arc::clone(&store);
        let s = ClippingStorage { store, tx };
        let hdl = std::thread::spawn(move || storage_main(rx, store_clone));
        (s, hdl)
    }

    /// Get a channel to add new clippings to the store
    pub fn add_clips(&self) -> Sender<Clipping> {
        self.tx.clone()
    }

    /// Get a channel that contains all the clippings stored so far
    pub fn get_clips(&self) -> Receiver<Clipping> {
        let (tx, rx) = channel();
        for clip in &*self.store.lock().expect("poisoned clipping sotre lock") {
            tx.send(clip.clone())
                .expect("could not send out the list of clippings");
        }
        rx
    }
}

/// Monitors the receiving channel, and add new clippings to the store
fn storage_main(rx: Receiver<Clipping>, mut store: Arc<Mutex<Vec<Clipping>>>) {
    use std::thread;
    use std::time::Duration;

    loop {
        match rx.recv() {
            Ok(clip) => push_clipping(clip, &mut store),
            Err(e) => error!("could not store clipping: {}", e),
        }
        thread::sleep(Duration::from_millis(1000));
    }
}

/// Acquire the lock to the store, and append the new clipping
fn push_clipping(clip: Clipping, store: &mut Arc<Mutex<Vec<Clipping>>>) {
    let store2 = Arc::get_mut(store)
        .expect("comeone else is holing the store")
        .get_mut()
        .expect("clipping storage lock got poisoned");
    store2.push(clip);
}
