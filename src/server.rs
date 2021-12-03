/// pub mod server: have a fn that can be started as a thread
/// simulates the game, shall be wrapped before the user

use crate::yard::{ YardSim, YardBuf, Direction };

use std::thread;
use std::time::Duration;
use std::sync::mpsc::{ Sender, Receiver, TryRecvError };
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum YardCtrl {
    NewSnake,
    CtrlSnake(u8, Direction),
}

#[derive(Serialize, Deserialize)]
pub enum YardInfo {
    RefreshScreen(YardBuf),
    RegisteredSnake(Option<u8>),
    Message(String),
}

/// simulating the yard in a seperate thread
/// use channel to input/output control, info and buffer
pub fn start_and_serve(
        info_tx: Sender<YardInfo>,
        ctrl_rx: Receiver<YardCtrl>,
    ) {
    // create a yard y and send the initial screen buffer
    let mut y = YardSim::new(30, 20, 5, 3);
    info_tx.send(YardInfo::RefreshScreen(y.generate_buf())).unwrap();
    loop {
        thread::sleep(Duration::from_millis(100));
        // receiving control signals
        loop {
            match ctrl_rx.try_recv() {
                Ok(YardCtrl::NewSnake) => {
                    info_tx.send(YardInfo::RegisteredSnake(y.init_snake())).unwrap();
                },
                Ok(YardCtrl::CtrlSnake(id, d)) => {
                    y.control_snake(id, d);
                },
                Err(TryRecvError::Empty) => { break; },
                Err(TryRecvError::Disconnected) => { return; },
            };
        }
        y.next_tick();
        info_tx.send(YardInfo::RefreshScreen(y.generate_buf())).unwrap();
    }
}