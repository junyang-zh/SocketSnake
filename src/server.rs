/// pub mod server: have a fn that can be started as a thread

use crate::yard::{ YardSim, YardBuf, Direction };

use std::thread;
use std::time::Duration;
use std::sync::mpsc::{ self, Sender, Receiver, TryRecvError };

pub enum YardCtrl {
    NewSnake,
    CtrlSnake(u8, Direction),
}

pub enum YardInfo {
    RegisteredSnake(Option<u8>),
    Message(String),
}

pub fn start_and_serve(
        buf_tx: Sender<YardBuf>,
        info_tx: Sender<YardInfo>,
        ctrl_rx: Receiver<YardCtrl>,
    ) {
    // create a yard y and send the initial screen buffer
    let mut y = YardSim::new(30, 20, 5, 3);
    buf_tx.send(y.generate_buf()).unwrap();
    loop {
        thread::sleep(Duration::from_millis(100));
        loop {
            match ctrl_rx.try_recv() {
                Ok(YardCtrl::NewSnake) => {
                    info_tx.send(YardInfo::RegisteredSnake(y.init_snake()));
                },
                Ok(YardCtrl::CtrlSnake(id, d)) => {
                    y.control_snake(id, d).unwrap();
                },
                Err(TryRecvError::Empty) => { break; },
                Err(TryRecvError::Disconnected) => panic!("Channel disconnected"),
            }
        }
        y.next_tick();
        buf_tx.send(y.generate_buf()).unwrap();
    }
}