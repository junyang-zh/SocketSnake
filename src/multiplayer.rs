/// pub mod multiplayer:
/// wrappers of client and server abstractions, gaming threads

use crate::{ server, client };
use crate::server::{ YardCtrl, YardInfo };
use crate::transmit::*;
use crate::{ tcp_recv, udp_recv };

use std::thread;
use std::sync::mpsc::{ self, TryRecvError };
use std::net::{ Shutdown, TcpListener, TcpStream, UdpSocket, Ipv4Addr };

pub const TCP_SERVER_PORT: &str = "127.0.0.1:14514";
pub const UDP_SERVER_PORT: &str = "0.0.0.0:19198";
pub const UDP_CLIENT_PORT: &str = "0.0.0.0:10114";
// multicast group can be from 234.0.2.0 to 238.255.255.255
pub const MULTICAST_GROUP_PORT: &str = "234.51.4.19:10114";
pub const MULTICAST_GROUP_ADDR: &Ipv4Addr = &Ipv4Addr::new(234, 51, 4, 19);

pub fn singleplayer_start(name: String) {
    // server sends to clients
    let (info_tx, info_rx) = mpsc::channel();
    // client sends to servers
    let (ctrl_tx, ctrl_rx) = mpsc::channel();

    let server_handle = thread::spawn(move || {
        server::start_and_serve(info_tx, ctrl_rx);
    });

    let client_handle = thread::spawn(move || {
        client::start_and_play(name, info_rx, ctrl_tx);
    });

    server_handle.join().unwrap_or(()); // Ok to SendError, client exits
    client_handle.join().unwrap();      // Not Ok to quit badly
}

pub fn handle_connection(ctrl_tx: mpsc::Sender<YardCtrl>, mut stream: TcpStream) {
    println!("Connected one client, establishing UDP connection");
    // send the multicast group for the client to join
    tcp_send(&mut stream, MULTICAST_GROUP_ADDR).unwrap();
    thread::spawn(move || {
        loop {
            let op: YardCtrl = match tcp_recv!(stream) {
                Ok(c) => { c },
                Err(e) => {
                    println!("Receiving failed {}", e);
                    break;
                },
            };
            match ctrl_tx.send(op.clone()) {
                Ok(_) => {
                    println!("Request handled: {:?}", op);
                },
                Err(_) => {
                    println!("Server quitted, ending TCP connection");
                    stream.shutdown(Shutdown::Both).expect("shutdown call failed");
                    break;
                }
            };
        }
    });
}

pub fn server_start() -> std::io::Result<()> {
    // server sends to clients
    let (info_tx, info_rx) = mpsc::channel();
    // client sends to servers
    let (ctrl_tx, ctrl_rx) = mpsc::channel();
    // start the backend
    let _server_handle = thread::spawn(move || {
        server::start_and_serve(info_tx, ctrl_rx);
    });

    // info from server (info_rx) always sent to UDP multicast
    let socket = UdpSocket::bind(UDP_SERVER_PORT).unwrap();
    thread::spawn(move || {
        loop {
            let info = match info_rx.recv() {
                Ok(c) => c,
                Err(_e) => break, // client quitted
            };
            udp_send(&socket, MULTICAST_GROUP_PORT, &info).unwrap();
            // printing information
            match info {
                YardInfo::RegisteredSnake(_, _) | YardInfo::Failed(_) => {
                    println!("Sent signal {:?}", info);
                },
                _ => {},
            }
        }
    });

    // communicate through TCP
    let listener = TcpListener::bind(TCP_SERVER_PORT)?;
    println!("Listening");
    // server wrapper listens for connection and send through channel
    loop {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    // spawn a child that sends ctrl to the backend
                    let ctrl_tx_clone = mpsc::Sender::clone(&ctrl_tx);
                    handle_connection(ctrl_tx_clone, stream);
                },
                Err(_e) => {},
            }
        }
    }
}

pub fn client_start(name: String, server_addr: String) {
    // server sends to clients
    let (info_tx, info_rx) = mpsc::channel();
    // client sends to servers
    let (ctrl_tx, ctrl_rx) = mpsc::channel();

    // sending ctrl signal using TCP
    println!("Connecting to {} ...", &server_addr);
    let mut stream = loop {
        match TcpStream::connect(&server_addr) {
            Ok(s) => { break s; },
            Err(e) => { println!("{}, retrying ...", e); },
        };
    };

    // when TCP connected, use TCP to receive a multicast group
    let group_addr: Ipv4Addr = tcp_recv!(stream).unwrap();
    // start listening UDP for buffer and information
    let socket = UdpSocket::bind(UDP_CLIENT_PORT).unwrap();
    socket.join_multicast_v4(&group_addr, &Ipv4Addr::UNSPECIFIED)
        .expect("Couldn't join multicast");

    // UDP listening thread, use channel to notify an end of service
    let (listener_kill, listener_killed) = mpsc::channel::<bool>();
    let listening_handle = thread::spawn(move || {
        loop {
            // child control
            match listener_killed.try_recv() {
                Ok(_) => { return; },
                Err(TryRecvError::Empty) => {},
                Err(TryRecvError::Disconnected) => { return; },
            }
            // doing work
            let server_info: YardInfo = match udp_recv!(&socket) {
                Ok(i) => { i }, // be silent to users
                Err(_) => { break; }, // client quitted
            };
            match info_tx.send(server_info) {
                Ok(_) => {}, // be silent to users
                Err(_) => { break; }, // client quitted
            };
        }
    });

    // TCP sending thread
    let (sender_kill, sender_killed) = mpsc::channel::<bool>();
    let sending_handle = thread::spawn(move || {
        loop {
            // child control
            match sender_killed.try_recv() {
                Ok(_) => { break; },
                Err(TryRecvError::Empty) => {},
                Err(TryRecvError::Disconnected) => { break; },
            }
            // doing work
            let ctrl = match ctrl_rx.recv() {
                Ok(c) => c,
                Err(_e) => break, // client quitted
            };
            tcp_send(&mut stream, &ctrl).unwrap();
        }
        // shutdown TCP connection
        stream.shutdown(Shutdown::Both).expect("Shutdown TCP connection failed");
    });

    client::start_and_play(name, info_rx, ctrl_tx); // note: will not return till end

    // if user ended playing, clean up the threads by just dropping the channel
    drop(listener_kill);
    drop(sender_kill);
    listening_handle.join().unwrap();
    sending_handle.join().unwrap();
}