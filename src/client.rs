/// pub mod client: have a fn that can be started as a thread
/// interact with the server and user

use crate::render;
use crate::server::{ YardCtrl, YardInfo };
use crate::yard::{ YardBuf, Direction };

use crossterm::event::{ poll, read, Event, KeyCode };

use std::time::Duration;
use std::sync::mpsc::{ self, Sender, Receiver, TryRecvError };

pub fn start_and_play(
        buf_rx: Receiver<YardBuf>,
        info_rx: Receiver<YardInfo>,
        ctrl_tx: Sender<YardCtrl>,
    ) {
    let mut ui = render::TUIHelper::new();
    ctrl_tx.send(YardCtrl::NewSnake);
    let id;
    match info_rx.recv().unwrap() {
        YardInfo::RegisteredSnake(given_id) => { id = given_id.unwrap(); },
        _ => { id = 0; panic!("Room may full") },
    }
    loop {
        loop {
            match buf_rx.try_recv() {
                Ok(buf) => {
                    ui.refresh_yard(buf).unwrap();
                },
                Err(TryRecvError::Empty) => { break; },
                Err(TryRecvError::Disconnected) => panic!("Channel disconnected"),
            }
        }
        /* still have bug
        loop {
            if poll(Duration::from_millis(10)).unwrap() {
                match read().unwrap() {
                    Event::Key(event) => {
                        match event.code {
                            KeyCode::Left => {
                                ctrl_tx.send(YardCtrl::CtrlSnake(id, Direction::L));
                            },
                            KeyCode::Right => {
                                ctrl_tx.send(YardCtrl::CtrlSnake(id, Direction::R));
                            },
                            KeyCode::Up => {
                                ctrl_tx.send(YardCtrl::CtrlSnake(id, Direction::U));
                            },
                            KeyCode::Down => {
                                ctrl_tx.send(YardCtrl::CtrlSnake(id, Direction::D));
                            },
                            KeyCode::Esc => { break; },
                            _ => {},
                        };
                    },
                    Event::Mouse(_event) => {},
                    Event::Resize(_width, _height) => {},
                }
            }
        }*/
    }
}