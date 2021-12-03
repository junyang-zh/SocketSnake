/// pub mod multiplayer:
/// wrappers of client and server abstractions, gaming threads

use crate::{ server, client };
use crate::server::{ YardCtrl, YardInfo };

use serde_yaml;

use std::thread;
use std::io::{ Read, Write };
use std::sync::mpsc;
use std::net::{ Shutdown, TcpListener, TcpStream, UdpSocket, Ipv4Addr };

// multicast group can be from 234.0.2.0 to 238.255.255.255
const MULTICAST_GROUP: &str = "234.51.4.19:19810";
const MULTICAST_GROUP_ADDR: &Ipv4Addr = &Ipv4Addr::new(234, 51, 4, 19);

pub fn singleplayer_start() {
    // server sends to clients
    let (info_tx, info_rx) = mpsc::channel();
    // client sends to servers
    let (ctrl_tx, ctrl_rx) = mpsc::channel();

    let server_handle = thread::spawn(move || {
        server::start_and_serve(info_tx, ctrl_rx);
    });

    let client_handle = thread::spawn(move || {
        client::start_and_play(info_rx, ctrl_tx);
    });

    server_handle.join().unwrap(); // never join, never stop, till panics
    client_handle.join().unwrap();
}

pub fn handle_connection(ctrl_tx: mpsc::Sender<YardCtrl>, mut stream: TcpStream) {
    thread::spawn(move || {
        loop {
            let mut buffer = [0; 1024];
            let len = match stream.read(&mut buffer) {
                    Ok(n) => n,
                    Err(_) => { break; },
                };
            if (len == 0) {
                continue;
            }
            let serialized: String = std::str::from_utf8(&buffer[0..len]).unwrap().to_string();
            let op: YardCtrl = serde_yaml::from_str(&serialized).unwrap();
            ctrl_tx.send(op).unwrap();
            println!("Request handled: {:?}", &serialized);
        }
        stream.shutdown(Shutdown::Both).expect("shutdown call failed");
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

    /// TODO
    // communicate through TCP
    let listener = TcpListener::bind("127.0.0.1:41919")?;
    println!("Listening");
    // server wrapper listens for connection and send through channel
    loop {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    // spawn a child that sends ctrl to the backend
                    let ctrl_tx_clone = mpsc::Sender::clone(&ctrl_tx);
                    handle_connection(ctrl_tx_clone, stream);
                    println!("Connected one client");
                },
                Err(_e) => {},
            }
        }
    }
}

pub fn client_start(server_addr: String) {
    // server sends to clients
    let (info_tx, info_rx) = mpsc::channel();
    // client sends to servers
    let (ctrl_tx, ctrl_rx) = mpsc::channel();
    
    let _client_handle = thread::spawn(move || {
        client::start_and_play(info_rx, ctrl_tx);
    });

    // receiving info (as well as screen buffer) through UDP
    let socket = UdpSocket::bind(MULTICAST_GROUP).unwrap();
    socket.join_multicast_v4(&MULTICAST_GROUP_ADDR, &Ipv4Addr::UNSPECIFIED)
        .expect("Couldn't join multicast");
    thread::spawn(move || {
        loop {
            let mut buffer = [0u8; 65535];
            socket.recv_from(&mut buffer).unwrap();
            let deserialized = String::from_utf8_lossy(&buffer[..]);
            let server_info: YardInfo = match serde_yaml::from_str(&deserialized) {
                    Ok(info) => info,
                    Err(_e) => continue,
                };
            info_tx.send(server_info).unwrap();
        }
    });

    // sending ctrl signal using TCP
    println!("Connecting to {} ...", &server_addr);
    let mut stream = loop {
        match TcpStream::connect(&server_addr) {
            Ok(s) => { break s; },
            Err(e) => { println!("{}, retrying ...", e); },
        };
    };
    thread::spawn(move || {
        let ctrl_rx_moved = ctrl_rx;
        loop {
            let serialized = match ctrl_rx_moved.recv() {
                Ok(ctrl) => match serde_yaml::to_string(&ctrl) {
                        Ok(s) => s,
                        Err(_e) => continue,
                    },
                Err(_e) => break,
            };
            stream.write(serialized.as_bytes()).unwrap();
            stream.flush().unwrap();
        }
    });
}