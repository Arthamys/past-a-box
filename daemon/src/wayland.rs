use crate::error::{Error, Result};
use dc::zwlr_data_control_manager_v1::RequestsTrait as DataControlManagerRequests;
use dc::zwlr_data_control_manager_v1::ZwlrDataControlManagerV1 as DataControlManager;
use dc::zwlr_data_control_offer_v1::RequestsTrait as OfferRequest;
use libc::pipe;
use wayland_client::protocol::wl_seat::WlSeat;
use wayland_client::{Display, EventQueue, GlobalManager, Proxy};
use wayland_protocols::wlr::unstable::data_control::v1::client as dc;

pub struct WaylandContext {
    display: Display,
    event_queue: EventQueue,
    dcm: Proxy<DataControlManager>,
    seat: Proxy<WlSeat>,
}

impl WaylandContext {
    /// Create a new wayland clipboard listener
    ///
    /// # Note
    /// This function will only work when ran under a wayland compositor that
    /// implements the zwlr_data_control protocol.
    /// This protocol is still experimental, but is supported by Sway.
    pub fn new() -> Result<WaylandContext> {
        let (display, mut event_queue) = Display::connect_to_env()?;
        let globals = GlobalManager::new(&display);
        event_queue.sync_roundtrip()?;
        let dcm = globals.instantiate_auto::<DataControlManager, _>(|ctrl_mgr| {
            ctrl_mgr.implement(
                |_, _| {
                    info!("Recieved data control manager event");
                },
                (),
            )
        })?;
        let seat = globals.instantiate_auto::<WlSeat, _>(|seat2| {
            seat2.implement(
                |_, _| {
                    info!("Seat handler");
                },
                (),
            )
        })?;
        Ok(WaylandContext {
            display,
            event_queue,
            dcm,
            seat,
        })
    }
}
