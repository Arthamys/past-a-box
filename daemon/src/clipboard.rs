use crate::wayland::WaylandContext;
use dc::zwlr_data_control_offer_v1::ZwlrDataControlOfferV1 as DataControlOffer;
use std::os::unix::io::RawFd;
use std::sync::{Mutex, MutexGuard};
use std::thread;
use wayland_client::EventQueue;
use wayland_protocols::wlr::unstable::data_control::v1::client as dc;

#[derive(Debug)]
pub struct Clipboard;

impl Clipboard {
    pub fn new_clipboard_thread() -> thread::JoinHandle<()> {
        thread::Builder::new()
            .name("clipboard listener".into())
            .spawn(|| {
                let mut ctx = WaylandContext::new().unwrap();
                ctx.register_handler(plaintext_handler);
                ctx.run();
            })
            .expect("Could not spawn clipboard listener thread")
    }
}

/// Holds state for a data offer transfer
#[derive(Debug)]
struct Transfer {
    ongoing: bool,
    pipes: (RawFd, RawFd),
}

impl Transfer {
    fn new() -> Self {
        Transfer {
            ongoing: false,
            pipes: (0, 0),
        }
    }
}

// create a thread local `&'static` variable
lazy_static! {
    static ref TRANSFER: Mutex<Transfer> = Mutex::new(Transfer::new());
}

fn plaintext_handler(offer: &DataControlOffer, mime_type: String) {
    if mime_type != String::from("text/plain") {
        return;
    }
    info!("Handling plaintext offer...");
    let mut guard = TRANSFER.lock().expect("could not acquire transfer lock");
    if guard.ongoing == false {
        start_transfer(offer, &mut guard);
    } else {
        transfer_data(&mut guard);
    }
}

fn start_transfer(offer: &DataControlOffer, transfer: &mut MutexGuard<Transfer>) {
    let mut fds = [0; 2];
    unsafe {
        let res = libc::pipe(fds.as_mut_ptr());
        if res != 0 {
            error!("Error creating pipe");
        }
    }
    transfer.pipes.0 = fds[0]; //read
    transfer.pipes.1 = fds[1]; //write
    info!(
        "pipes: [read: {}] [write: {}]",
        transfer.pipes.0, transfer.pipes.1
    );
    offer.receive(String::from("text/plain"), transfer.pipes.1);
    transfer.ongoing = true;
    info!("asked to receive the content of the clipboard");
}

fn transfer_data(transfer: &mut MutexGuard<Transfer>) {
    let pipe_fds = transfer.pipes.clone();
    // start a detached thread to read incoming data from the server
    let hdl = thread::Builder::new()
        .name("transfer_thread".into())
        .spawn(move || {
            let mut clip = String::new();
            let mut buf: [u8; 255] = [0; 255];
            info!("Transfering data");
            unsafe {
                let ptr = buf.as_mut_ptr();
                let mut count = 0;

                while {
                    count = libc::read(pipe_fds.0, ptr as *mut libc::c_void, 255);
                    if count < 0 {
                        error!("read syscall failed");
                    }
                    let part = String::from_utf8(buf.to_vec()).unwrap();
                    clip.push_str(&part);
                    for c in &mut buf[0..255] {
                        *c = 0;
                    }
                    count == 255
                } {}
                if libc::close(pipe_fds.0) < 0 || libc::close(pipe_fds.1) < 0 {
                    error!("Could not close pipe");
                }
            }
            info!("received data: {}", clip);
        })
        .unwrap();

    transfer.ongoing = false;
}
