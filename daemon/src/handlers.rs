use crate::clipboard::Clipboard;
use api::common::clipping::Clipping;
use data_control::zwlr_data_control_device_v1::ZwlrDataControlDeviceV1;
use data_control::zwlr_data_control_offer_v1::{self, ZwlrDataControlOfferV1};
use data_control::zwlr_data_control_source_v1::{self, ZwlrDataControlSourceV1};
use os_pipe::pipe;
use std::cell::RefCell;
use std::io::Read;
use std::os::unix::io::{AsRawFd, RawFd};
use std::rc::Rc;
use std::sync::mpsc::Sender;
use std::thread;
use wayland_client::protocol::wl_seat::{self, WlSeat};
use wayland_client::NewProxy;
use wayland_client::{Display, EventQueue};
use wayland_protocols::wlr::unstable::data_control::v1::client as data_control;

// Handler for the WlSeat events
pub struct WlSeatHandler;

impl wl_seat::EventHandler for WlSeatHandler {
    fn name(&mut self, object: WlSeat, name: String) {
        info!("Received event for seat {}", name);
    }
}

// Handler for the DataControlDevice events
pub struct DataDeviceHandler;

impl data_control::zwlr_data_control_device_v1::EventHandler for DataDeviceHandler {
    fn data_offer(
        &mut self,
        object: ZwlrDataControlDeviceV1,
        offer: NewProxy<ZwlrDataControlOfferV1>,
    ) {
        info!("New data offer");
        if let Some(clipb) = object.as_ref().user_data::<Rc<RefCell<Clipboard>>>() {
            info!("implemented the data control offer handler");
            offer.implement(DataControlOfferHandler, clipb.clone());
        } else {
            info!("No clipboard set");
        }
    }

    fn selection(&mut self, object: ZwlrDataControlDeviceV1, id: Option<ZwlrDataControlOfferV1>) {
        if let None = id {
            info!("No new selection...");
            return;
        }
        info!("New selection");
        transfer_selection(&id.unwrap(), String::from("text/plain;charset=utf-8"));
    }

    fn finished(&mut self, object: ZwlrDataControlDeviceV1) {
        info!("DataControlDevice is now invalid");
    }

    fn primary_selection(
        &mut self,
        object: ZwlrDataControlDeviceV1,
        id: Option<ZwlrDataControlOfferV1>,
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
                if let None = offer.as_ref().user_data::<Rc<RefCell<Clipboard>>>() {
                    error!("No channel for this offer");
                }
                //let clip = transfer_selection(&offer, mime_type);

                /*offer*/
                //.as_ref()
                //.user_data::<Sender<Clipping>>()
                //.unwrap()
                /*.send(clip);*/
                //info!("sent clipping");
                //self.user_data.transfer.read();
                //receive_contents(offer);
                //offer.receive();
                //queue.sync_roundtrip();
            }
            _ => (),
        }
    }
}

// get the data of the selection as a clipping
fn transfer_selection(offer: &ZwlrDataControlOfferV1, mime_type: String) -> Clipping {
    static mut toggle: u32 = 1;
    let mut clipb = offer
        .as_ref()
        .user_data::<Rc<RefCell<Clipboard>>>()
        .expect("No clipboard user_data set for transfer_selection")
        .borrow_mut();

    unsafe {
        if toggle % 2 == 1 {
            let (reader, writer) = pipe().expect("Could not create pipe");
            //info!("pipe fds: rx {:?}, wx {:?}", &reader, &writer);
            clipb.transfer.set_reader(reader);

            offer.receive(mime_type, writer.as_raw_fd());
            //PROSPER
            //offer.receive(mime_type, 1);
            drop(writer);
            let mut event_queue = get_event_queue();
            event_queue.sync_roundtrip().unwrap();
            toggle = toggle + 1;
            Clipping(String::new())
        } else {
            //let reader = clipb.transfer.get_reader();
            //info!("reading from fd: {:?}", reader);
            //copy_data(reader, &clipb.chan());
            toggle = toggle + 1;
            //let mut rsp = vec![];
            let mut rsp = String::new();
            clipb.transfer.read_to_string(&mut rsp);
            //reader.read_to_string(&mut rsp);
            info!("Selection {:?}", &rsp);
            Clipping(rsp)
            // TODO: clear transfer
            //Clipping(String::from_utf8(rsp).unwrap())
        }
    }
}

fn get_event_queue() -> EventQueue {
    let (display, mut event_queue) =
        Display::connect_to_env().expect("Could not connect to display");
    event_queue
}

pub struct DataSourceHandler {}

impl zwlr_data_control_source_v1::EventHandler for DataSourceHandler {
    fn send(&mut self, source: ZwlrDataControlSourceV1, _mime_type: String, target_fd: RawFd) {
        info!("Send");
        let clipboard = source.as_ref().user_data::<Clipboard>().unwrap();
    }
}

fn copy_data(read_fd: RawFd, tx: &Sender<Clipping>) {
    // start a detached thread to read incoming data from the server
    let mut clip = String::new();
    let mut buf: [u8; 255] = [0; 255];
    info!("Copying data");
    unsafe {
        let ptr = buf.as_mut_ptr();
        let mut count;

        while {
            info!("zob, fd: {}", read_fd);
            count = libc::read(read_fd, ptr as *mut libc::c_void, 255);
            if count < 0 {
                error!("read syscall failed");
            }
            info!("read {} bytes", count);
            let part = String::from_utf8(buf.to_vec()).unwrap();
            clip.push_str(&part);
            for c in &mut buf[0..255] {
                *c = 0;
            }
            count == 255
        } {}
    }
    info!("received data: {}", &clip);
    tx.send(Clipping(clip));
}
