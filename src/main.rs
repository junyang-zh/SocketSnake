pub mod yard;
pub mod render;
pub mod server;
pub mod client;

use std::thread;
use std::sync::mpsc;

fn main() {
    // server sends to clients
    let (buf_tx, buf_rx) = mpsc::channel();
    let (info_tx, info_rx) = mpsc::channel();
    // client sends to servers
    let (ctrl_tx, ctrl_rx) = mpsc::channel();

    let server_handle = thread::spawn(move || {
        server::start_and_serve(buf_tx, info_tx, ctrl_rx);
    });

    let client_handle = thread::spawn(move || {
        client::start_and_play(buf_rx, info_rx, ctrl_tx);
    });

    server_handle.join().unwrap(); // never join, never stop, till panics
    client_handle.join().unwrap();
    // say goodbye
    
}
