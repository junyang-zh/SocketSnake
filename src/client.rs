/// pub mod client: have a fn that can be started as a thread
/// interact with the server and user

use crate::render;
use crate::server::{ YardCtrl, YardInfo };
use crate::yard::{ YardBuf, Direction };

use crossterm::event::{ poll, read, Event, KeyCode };

use std::thread;
use std::time::Duration;
use std::sync::mpsc::{ Sender, Receiver, TryRecvError };

pub fn start_and_play(
        buf_rx: Receiver<YardBuf>,
        info_rx: Receiver<YardInfo>,
        ctrl_tx: Sender<YardCtrl>,
    ) {
    let mut ui = render::TUIHelper::new();
    ctrl_tx.send(YardCtrl::NewSnake).unwrap();
    let id;
    match info_rx.recv().unwrap() {
        YardInfo::RegisteredSnake(given_id) => { id = given_id.unwrap(); },
        _ => { panic!("Room may full") },
    }
    let _refresing_handle = thread::spawn(move || {
        // receiving and printing buf
        loop {
            thread::sleep(Duration::from_millis(10));
            match buf_rx.try_recv() {
                Ok(buf) => {
                    ui.refresh_yard(buf).unwrap();
                },
                Err(TryRecvError::Empty) => {},
                Err(TryRecvError::Disconnected) => { return; },
            }
        }
    });
    loop {
        // polling keyboard strike
        if poll(Duration::from_millis(10)).unwrap() {
            match read().unwrap() {
                Event::Key(event) => {
                    match event.code {
                        KeyCode::Left => {
                            ctrl_tx.send(YardCtrl::CtrlSnake(id, Direction::L)).unwrap();
                        },
                        KeyCode::Right => {
                            ctrl_tx.send(YardCtrl::CtrlSnake(id, Direction::R)).unwrap();
                        },
                        KeyCode::Up => {
                            ctrl_tx.send(YardCtrl::CtrlSnake(id, Direction::U)).unwrap();
                        },
                        KeyCode::Down => {
                            ctrl_tx.send(YardCtrl::CtrlSnake(id, Direction::D)).unwrap();
                        },
                        KeyCode::Esc => { return; },
                        _ => {},
                    };
                },
                Event::Mouse(_event) => {},
                Event::Resize(_width, _height) => {},
            }
        }
    }
}