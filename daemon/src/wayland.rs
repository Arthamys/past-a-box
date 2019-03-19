use crate::clipboard::Clipboard;
use crate::error::Result;
use crate::Daemon;
use api::common::clipping::Clipping;
use dc::zwlr_data_control_manager_v1::ZwlrDataControlManagerV1 as DataControlManager;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use wayland_client::protocol::wl_seat::WlSeat;
use wayland_client::{Display, EventQueue, GlobalManager};
use wayland_protocols::wlr::unstable::data_control::v1::client as dc;

pub struct WaylandContext {
    display: Display,
    event_queue: EventQueue,
    dcm: DataControlManager,
    seat: WlSeat,
    rx: Receiver<Clipping>,
}

impl WaylandContext {
    /// Create a new wayland clipboard listener
    ///
    /// # Note
    /// This function will only work when ran under a wayland compositor that
    /// implements the zwlr_data_control protocol.
    /// This protocol is still experimental, but is supported by Sway.
    pub fn new(
        clipboard: Rc<RefCell<Clipboard>>,
        rx: Receiver<Clipping>,
    ) -> Result<WaylandContext> {
        use crate::handlers::{DataDeviceHandler, WlSeatHandler};

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
            dev.implement(DataDeviceHandler, clipboard)
        })
        .expect("could not register data_device handler");
        info!("Handler registered");

        info!("wayland context created");
        Ok(WaylandContext {
            display,
            event_queue,
            dcm,
            seat,
            rx,
        })
    }

    /// Start the event loop
    pub fn run(&mut self, s: &'static Mutex<Daemon>) -> ! {
        use std::{thread, time};

        let freq = time::Duration::from_millis(1000);
        info!("starting the event_loop");
        loop {
            self.event_queue.sync_roundtrip().unwrap();
            info!("Reading clippings");
            let c = self.rx.try_recv();
            if c.is_ok() {
                info!("Read clipping: {:?}", &c);
                s.lock().unwrap().add_clipping(c.unwrap());
            }
            thread::sleep(freq);
        }
    }
}
