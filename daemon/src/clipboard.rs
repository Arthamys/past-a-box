use crate::wayland::WaylandContext;
use crate::Daemon;
use api::common::clipping::Clipping;
use dc::zwlr_data_control_offer_v1::ZwlrDataControlOfferV1 as DataControlOffer;
use os_pipe::{pipe, PipeReader, PipeWriter};
use std::cell::RefCell;
use std::os::unix::io::{AsRawFd, RawFd};
use std::rc::Rc;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use wayland_protocols::wlr::unstable::data_control::v1::client as dc;

#[derive(Debug)]
pub struct Clipboard {
    snd: Sender<Clipping>,
    pub transfer: Transfer,
}

impl Clipboard {
    pub fn new() -> (Clipboard, Receiver<Clipping>) {
        let (tx, rx) = channel();
        (
            Clipboard {
                snd: tx,
                transfer: Transfer::new(),
            },
            rx,
        )
    }

    pub fn new_clipboard_thread(s: &'static Mutex<Daemon>) -> thread::JoinHandle<()> {
        thread::Builder::new()
            .name("clipboard listener".into())
            .spawn(move || {
                let (clipboard, rx) = Clipboard::new();
                let mut ctx = WaylandContext::new(Rc::new(RefCell::new(clipboard)), rx).unwrap();
                ctx.run(s);
            })
            .expect("Could not spawn clipboard listener thread")
    }

    pub fn chan(&self) -> Sender<Clipping> {
        self.snd.clone()
    }
}

/// Holds state for a data offer transfer
#[derive(Debug)]
pub struct Transfer {
    ongoing: bool,
    pipe: Option<PipeReader>,
}

impl Transfer {
    pub fn new() -> Self {
        Transfer {
            ongoing: false,
            pipe: None,
        }
    }

    /// Set the reader end of the pipe for this transfer
    pub fn set_reader(&mut self, fd: PipeReader) {
        self.pipe = Some(fd);
    }

    /// get the reader en of the pipe for this transfer
    pub fn get_reader(&self) -> RawFd {
        match &self.pipe {
            Some(p) => p.as_raw_fd(),
            _ => 1,
        }
    }

    pub fn read_to_string(&mut self, st: &mut String) {
        use std::io::Read;
        match &mut self.pipe {
            Some(p) => p.read_to_string(st).expect("Could not read to string"),
            _ => 0,
        };
    }
}

// create a thread local `&'static` variable
lazy_static! {
    static ref TRANSFER: Mutex<Transfer> = Mutex::new(Transfer::new());
}

fn plaintext_handler(s: &Arc<Mutex<Vec<Clipping>>>, offer: &DataControlOffer, mime_type: String) {
    if mime_type != String::from("text/plain") {
        return;
    }
    info!("Handling plaintext offer...");
    let mut guard = TRANSFER.lock().expect("could not acquire transfer lock");
    if guard.ongoing == false {
        start_transfer(offer, &mut guard);
    } else {
        transfer_data(&s, &mut guard);
    }
}

fn start_transfer(offer: &DataControlOffer, transfer: &mut MutexGuard<Transfer>) {
    let (reader, writer) = pipe().unwrap();
    // close the write end
    offer.receive(String::from("text/plain"), writer.as_raw_fd());
    drop(writer);
    transfer.pipe = Some(reader);
    transfer.ongoing = true;
    info!("asked to receive the content of the clipboard");
}

fn transfer_data(s: &Arc<Mutex<Vec<Clipping>>>, transfer: &mut MutexGuard<Transfer>) {
    /*// start a detached thread to read incoming data from the server*/
    //thread::Builder::new()
    //handle: Box<FnMut(&dc::zwlr_data_control_offer_v1::ZwlrDataControlOfferV1, String)>,
    //.name("transfer_thread".into())
    //.spawn(move || {
    //let mut clip = String::new();
    //let mut buf: [u8; 255] = [0; 255];
    //info!("Transfering data");
    //unsafe {
    //let ptr = buf.as_mut_ptr();
    //let mut count;

    //while {
    //count = libc::read(pipe_fds.0, ptr as *mut libc::c_void, 255);
    //if count < 0 {
    //error!("read syscall failed");
    //}
    //let part = String::from_utf8(buf.to_vec()).unwrap();
    //clip.push_str(&part);
    //for c in &mut buf[0..255] {
    //*c = 0;
    //}
    //count == 255
    //} {}
    //if libc::close(pipe_fds.0) < 0 || libc::close(pipe_fds.1) < 0 {
    //error!("Could not close pipe");
    //}
    //}
    //info!("received data: {}", &clip);
    //let mut clips = s.get_mut().expect("could not lock clippings");
    //clips.push(Clipping(clip));
    //})
    //.unwrap();

    transfer.ongoing = false;
    transfer.pipe = None;
}
