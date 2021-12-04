/// pub mod server: have a fn that can be started as a thread
/// simulates the game, shall be wrapped before the user

use crate::yard::{ self, YardSim, YardBuf, Direction };
use crate::render::Color;

use std::thread;
use std::time::Duration;
use std::sync::mpsc::{ Sender, Receiver, SendError, TryRecvError };
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum YardCtrl {
    // send a random client identifier and name for the server to shake hands
    NewSnake(u64, String),
    CtrlSnake(u64, Direction),
}

pub type BoardType = Vec<(Color, String)>; // board can be rendered with color

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum YardInfo {
    // send back the handle for the client, and request status
    RegisteredSnake(u64, bool),
    RefreshScreen(YardBuf),
    Failed(u64),
    Board(BoardType),
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
    let mut field_id = HashMap::new();  // client id to field id
    // field id to client id & name
    let mut client_id = [None; yard::MAX_PLAYERS as usize];
    let mut client_name = [None, None, None, None, None]; // again
    loop {
        thread::sleep(Duration::from_millis(100));
        // receiving control signals
        loop {
            match ctrl_rx.try_recv() {
                Ok(YardCtrl::NewSnake(rid, name)) => { // register snake
                    match y.init_snake() {
                        Some(id) => {
                            field_id.insert(rid, id);
                            client_id[id as usize] = Some(rid);
                            client_name[id as usize] = Some(name);
                            match info_tx.send(YardInfo::RegisteredSnake(rid, true)) {
                                Ok(_) => {}, Err(_) => { return; },
                            };
                        },
                        None => {

                        },
                    }
                },
                Ok(YardCtrl::CtrlSnake(id, d)) => {
                    match field_id.get(&id) {
                        Some(i) => { y.control_snake(*i, d); },
                        None => { 
                            println!("Backend received a ctrl of dead snake");
                            continue;
                        },
                    }
                },
                Err(TryRecvError::Empty) => { break; },
                Err(TryRecvError::Disconnected) => { return; },
            };
        }
        let (score, failed) = y.next_tick();
        let mut board = BoardType::new();
        for i in 0..yard::MAX_PLAYERS as usize {
            if score[i] > 0 { // there is a snake i
                board.push((
                    yard::PLAYER_COLOR_MAP[i],
                    format!("{}: {}\n", client_name[i].as_ref().unwrap(), score[i]),
                ));
            }
            if failed[i] {
                match info_tx.send(YardInfo::Failed(client_id[i].unwrap())) {
                    Ok(_) => {}, Err(_) => { return; },
                };
                field_id.remove(&client_id[i].unwrap());
                client_id[i] = None;
                client_name[i] = None;
            }
        }
        match info_tx.send(YardInfo::Board(board)) {
            Ok(_) => {}, Err(_) => { return; },
        };
        match info_tx.send(YardInfo::RefreshScreen(y.generate_buf())) {
            Ok(_) => {}, Err(_) => { return; },
        };
    }
}