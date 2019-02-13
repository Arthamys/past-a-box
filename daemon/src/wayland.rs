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
        event_queue.sync_roundtrip()?;
        Ok(WaylandContext {
            display,
            event_queue,
            dcm,
            seat,
        })
    }

    /// Register the clipboard data handler
    pub fn register_handler(
        &mut self,
        handle: fn(&Proxy<dc::zwlr_data_control_offer_v1::ZwlrDataControlOfferV1>, String),
    ) {
        self.dcm
            .get_data_device(&self.seat, |clipboard| {
                clipboard.implement(
                    |clip_evt, _| {
                        use dc::zwlr_data_control_device_v1::Event;
                        match clip_evt {
                            Event::Selection { id } => info!("new selection: {}", id.is_some()),

                            Event::Finished => info!("data control manager should be destroyed."),

                            Event::DataOffer { id } => {
                                // implement_closure to capture state (event_queue)
                                id.implement(
                                    |event, dco /*data control offer*/| {
                                        use dc::zwlr_data_control_offer_v1::Event as OfferEvent;
                                        match event {
                                            OfferEvent::Offer { mime_type } => {
                                                // look for supported mime_type
                                                if mime_type == "text/plain" {
                                                    //receive the data from the clipboard
                                                    //create pipe and get it's RawFd
                                                    handle(&dco, String::from("text/plain"));
                                                } else {
                                                    info!(
                                                        "Got offer for unsupoprted type [{}]",
                                                        mime_type
                                                    )
                                                }
                                            }
                                        }
                                    },
                                    (),
                                );
                            }
                        }
                    },
                    (),
                )
            })
            .expect("could not get data device");
        self.event_queue.sync_roundtrip().unwrap();
    }
}
