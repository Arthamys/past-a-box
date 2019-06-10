use clipboard::ClipboardContext;
use clipboard::ClipboardProvider;
use data_control::zwlr_data_control_device_v1::ZwlrDataControlDeviceV1;
use data_control::zwlr_data_control_offer_v1::{self, ZwlrDataControlOfferV1};
use data_control::zwlr_data_control_source_v1::{self, ZwlrDataControlSourceV1};
use std::os::unix::io::RawFd;
use wayland_client::protocol::wl_seat::{self, WlSeat};
use wayland_client::NewProxy;
use wayland_client::{Display, EventQueue};
use wayland_protocols::wlr::unstable::data_control::v1::client as data_control;

// Handler for the WlSeat events
pub struct WlSeatHandler;

impl wl_seat::EventHandler for WlSeatHandler {
    fn name(&mut self, _object: WlSeat, name: String) {
        info!("Received event for seat {}", name);
    }
}

// Handler for the DataControlDevice events
pub struct DataDeviceHandler;

impl data_control::zwlr_data_control_device_v1::EventHandler for DataDeviceHandler {
    /// data_offer introduces the DataControlOffer object that is used to receive
    /// the data that is advertised.
    fn data_offer(
        &mut self,
        object: ZwlrDataControlDeviceV1,
        offer: NewProxy<ZwlrDataControlOfferV1>,
    ) {
        info!("New data offer");
        offer.implement(DataControlOfferHandler, ());
    }

    fn selection(&mut self, _object: ZwlrDataControlDeviceV1, id: Option<ZwlrDataControlOfferV1>) {
        if let None = id {
            info!("No new selection...");
            return;
        }
        let id = id.unwrap();
        info!("New selection");
    }

    fn finished(&mut self, _object: ZwlrDataControlDeviceV1) {
        info!("DataControlDevice is now invalid");
    }

    fn primary_selection(
        &mut self,
        _object: ZwlrDataControlDeviceV1,
        _id: Option<ZwlrDataControlOfferV1>,
    ) {
        info!("New primary selection");
    }
}

// Handler for DataControlOffer events
pub struct DataControlOfferHandler;

impl zwlr_data_control_offer_v1::EventHandler for DataControlOfferHandler {
    fn offer(&mut self, offer: ZwlrDataControlOfferV1, mime_type: String) {
        // TODO: Filter for supported mime types here, and receive the plaintext data
        match mime_type.as_ref() {
            "text/plain;charset=utf-8" => {
                info!("Handling text utf8 offer");
                let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
                let clip = ctx.get_contents();
                info!("content of clipboard: {:?}", &clip);
            }
            _ => (),
        }
    }
}

fn get_event_queue() -> EventQueue {
    let (_display, event_queue) = Display::connect_to_env().expect("Could not connect to display");
    event_queue
}

pub struct DataSourceHandler {}

impl zwlr_data_control_source_v1::EventHandler for DataSourceHandler {
    fn send(&mut self, source: ZwlrDataControlSourceV1, _mime_type: String, target_fd: RawFd) {
        info!("Data source handler send");
    }
}
