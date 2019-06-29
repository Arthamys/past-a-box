use crate::error::Result;
use crate::DAEMON;
use clipboard::ClipboardContext;
use clipboard::ClipboardProvider;
use dc::zwlr_data_control_manager_v1::ZwlrDataControlManagerV1 as DataControlManager;
use wayland_client::protocol::wl_seat::WlSeat;
use wayland_client::{Display, EventQueue, GlobalManager};
use wayland_protocols::wlr::unstable::data_control::v1::client as dc;

pub struct WaylandContext {
    event_queue: EventQueue,
}

impl WaylandContext {
    /// Create a new wayland clipboard listener
    ///
    /// # Note
    /// This function will only work when ran under a wayland compositor that
    /// implements the zwlr_data_control protocol.
    /// This protocol is still experimental, but is supported by Sway.
    pub fn new() -> Result<WaylandContext> {
        use crate::handlers::WlSeatHandler;

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
        let seat =
            globals.instantiate_exact::<WlSeat, _>(1, |seat| seat.implement(WlSeatHandler, ()))?;

        info!("registering DataDevice Handler...");
        dcm.get_data_device(&seat, move |dev| {
            dev.implement_closure(
                |evt, _ /*data_control_device*/| {
                    use dc::zwlr_data_control_device_v1::Event;
                    match evt {
                        Event::DataOffer { id } => {
                            id.implement_closure(
                                |offer, _ /*data_control_device*/| {
                                    use dc::zwlr_data_control_offer_v1::Event;
                                    match offer {
                                        Event::Offer { ref mime_type }
                                            if mime_type == "text/plain;charset=utf-8" =>
                                        {
                                            let mut ctx: ClipboardContext =
                                                ClipboardProvider::new().unwrap();
                                            let clip = ctx.get_contents();
                                            let d = DAEMON.lock().unwrap();
                                            let mut d2 = d.storage.lock().unwrap();
                                            let clip = clip.unwrap().into();
                                            //check if clipping already exists
                                            if d2.contains(&clip) == false {
                                                d2.push(clip);
                                            }
                                            // append to cliping storage
                                        }
                                        Event::Offer { mime_type } => {
                                            info!("Received offer for mime_type {}", mime_type)
                                        }
                                        _ => unreachable!(),
                                    }
                                },
                                (),
                            );
                        }
                        Event::Finished => info!("Data device is getting destroyed"),
                        Event::Selection { id } => {
                            if id.is_some() {
                                info!("Received Selection");
                            } else {
                                info!("No Selection before start");
                            }
                        }
                        _ => unimplemented!(),
                    }
                },
                (),
            )
        })
        .expect("could not register data_device handler");
        info!("Handler registered");

        info!("wayland context created");
        Ok(WaylandContext { event_queue })
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
