use wayland_client::protocol::wl_seat::{self, WlSeat};

// Handler for the WlSeat events
pub struct WlSeatHandler;

impl wl_seat::EventHandler for WlSeatHandler {
    fn name(&mut self, _object: WlSeat, name: String) {
        info!("Received event for seat {}", name);
    }
}
