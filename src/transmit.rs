/// pub mod transmit:
/// wrapped TCP and UDP scaffolds, send serialized and recv parsed

use serde_yaml;
use serde::{Deserialize, Serialize};

pub use std::io::{ Read, Write, Result, Error };
use std::net::{ Shutdown, TcpListener, TcpStream, UdpSocket, SocketAddr };

pub const TCP_BUFFER_SIZE: usize = 1024;
pub const UDP_BUFFER_SIZE: usize = 65507;

pub fn tcp_send<T: Serialize>(stream: &mut TcpStream, obj: &T) -> Result<()> {
    let serialized: String = serde_yaml::to_string(&obj).unwrap();
    stream.write(serialized.as_bytes())?;
    stream.flush()?;
    Ok(())
}

/// since Deserialize has a lifetime that can't be properly tackled without verbose
/// use macro instead
#[macro_export]
macro_rules! tcp_recv{
    ($stream: expr) => {
        {
            let mut buffer = [0; TCP_BUFFER_SIZE];
            let mut len = 0;
            loop {
                let inc_len = match $stream.read(&mut buffer[len..]) {
                    Ok(c) => c,
                    Err(e) => { break Err(e); },
                };
                len += inc_len;
                match serde_yaml::from_slice(&buffer[..len]) {
                    Ok(obj) => { break Ok(obj); },
                    Err(_) => { continue; },
                };
            }
        }
    }
}

pub fn udp_send<T: Serialize>(socket: &UdpSocket, src: &str, obj: &T) -> Result<()> {
    let serialized = serde_yaml::to_string(&obj).unwrap();
    socket.send_to(serialized.as_bytes(), src)?;
    Ok(())
}

#[macro_export]
macro_rules! udp_recv{
    ($socket: expr) => {
        {
            let mut buffer = [0; UDP_BUFFER_SIZE];
            let mut len = 0;
            loop {
                let (amt, _src) = match $socket.recv_from(&mut buffer[len..]) {
                    Ok(t) => t,
                    Err(e) => { break Err(e); },
                };
                len += amt;
                match serde_yaml::from_slice(&buffer[..len]) {
                    Ok(obj) => { break Ok(obj); },
                    Err(_) => { continue; },
                };
            }
        }
    }
}