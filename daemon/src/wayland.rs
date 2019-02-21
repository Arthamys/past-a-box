use crate::error::Result;
use dc::zwlr_data_control_manager_v1::ZwlrDataControlManagerV1 as DataControlManager;
use wayland_client::protocol::wl_seat::WlSeat;
use wayland_client::{Display, EventQueue, GlobalManager};
use wayland_protocols::wlr::unstable::data_control::v1::client as dc;

pub struct WaylandContext {
    display: Display,
    event_queue: EventQueue,
    dcm: DataControlManager,
    seat: WlSeat,
}

impl WaylandContext {
    /// Create a new wayland clipboard listener
    ///
    /// # Note
    /// This function will only work when ran under a wayland compositor that
    /// implements the zwlr_data_control protocol.
    /// This protocol is still experimental, but is supported by Sway.
    pub fn new() -> Result<WaylandContext> {
        info!("Creating wayland context...");
        let (display, mut event_queue) = Display::connect_to_env()?;
        let globals = GlobalManager::new(&display);
        event_queue.sync_roundtrip()?;

        let dcm = globals.instantiate_exact::<DataControlManager, _>(1, |ctrl_mgr| {
            ctrl_mgr.implement_closure(
                |_, _| {
                    info!("Recieved data control manager event");
                },
                (),
            )
        })?;
        let seat = globals.instantiate_exact::<WlSeat, _>(1, |seat| {
            seat.implement_closure(
                move |_ /*event*/, _ /*seat*/| {
                    info!("Seat handler");
                },
                (),
            )
        })?;

        //event_queue.sync_roundtrip()?;

        info!("wayland context created");
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
        handle: fn(&dc::zwlr_data_control_offer_v1::ZwlrDataControlOfferV1, String),
    ) {
        info!("registering new handler...");
        let mut handler_eq = self.display.create_event_queue();
        self.dcm
            .get_data_device(&self.seat, move |clipboard| {
                clipboard.implement_closure(
                    move |clip_evt, _| {
                        use dc::zwlr_data_control_device_v1::Event;
                        match clip_evt {
                            Event::Selection { id } => info!("new selection: {}", id.is_some()),

                            Event::Finished => info!("data control manager should be destroyed."),

                            Event::DataOffer { id } => {
                                use dc::zwlr_data_control_offer_v1::Event as OfferEvent;
                                id.implement_closure(
                                    move |event, dco| match event {
                                        OfferEvent::Offer { mime_type } => {
                                            info!(
                                            "Got offer for clipboard content under mime type [{}]",
                                            mime_type
                                        );
                                            handle(&dco, mime_type);
                                        }
                                        OfferEvent::__nonexhaustive => unreachable!(),
                                    },
                                    (),
                                );
                            }

                            Event::PrimarySelection { id } => {
                                if let Some(_offer) = id {
                                    info!("Got primary selection offer");
                                } else {
                                    info!("Spontaneous PrimarySelection event");
                                }
                            }

                            Event::__nonexhaustive => unreachable!(),
                        }
                    },
                    (),
                )
            })
            .expect("could not get data device");
        //self.event_queue.sync_roundtrip().unwrap();
        info!("Handler registered");
    }

    /// Start the event loop
    pub fn run(&mut self) -> ! {
        use std::{thread, time};

        let freq = time::Duration::from_millis(1000);
        info!("starting the event_loop");
        loop {
            self.event_queue.sync_roundtrip().unwrap();
            thread::sleep(freq);
        }
    }
}
