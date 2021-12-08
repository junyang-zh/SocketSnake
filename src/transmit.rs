/// pub mod transmit:
/// wrapped TCP and UDP scaffolds, send serialized and recv parsed

use serde::{ Serialize };

pub use std::io::{ Read, Write, Result, Error, ErrorKind };
use std::net::{ TcpStream, UdpSocket };

pub const TCP_BUFFER_SIZE: usize = 1024;
pub const UDP_BUFFER_SIZE: usize = 65507;

pub fn tcp_send<T: Serialize>(stream: &mut TcpStream, obj: &T) -> Result<()> {
    let serialized: Vec<u8> = bincode::serialize(&obj).unwrap();
    stream.write(&serialized)?;
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
                    Ok(l) => l,
                    Err(e) => { break Err(e); },
                };
                len += inc_len;
                match bincode::deserialize(&buffer[..len]) {
                    Ok(obj) => { break Ok(obj); },
                    _ => { continue; },
                };
            }
        }
    }
}

pub fn udp_send<T: Serialize>(socket: &UdpSocket, src: &str, obj: &T) -> Result<()> {
    let serialized: Vec<u8> = bincode::serialize(&obj).unwrap();
    socket.send_to(&serialized, src)?;
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
                match bincode::deserialize(&buffer[..len]) {
                    Ok(obj) => { break Ok(obj); },
                    Err(_) => { continue; },
                };
            }
        }
    }
}