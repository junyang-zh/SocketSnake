/// pub mod client: have a fn that can be started as a thread
/// interact with the server and user

use crate::render;
use crate::server::{ YardCtrl, YardInfo };
use crate::yard::Direction;

use crossterm::event::{ poll, read, Event, KeyCode };
use rand::{ thread_rng, Rng };

use std::thread;
use std::time::Duration;
use std::sync::mpsc::{ Sender, Receiver, TryRecvError };

/// polling keyboard strike and customize control
/// id: client id, ctrl_tx: control signal channel
pub fn polling_keyboard(id: u64, ctrl_tx: Sender<YardCtrl>) {
    loop {
        if poll(Duration::from_millis(10)).unwrap() {
            match read().unwrap() {
                Event::Key(event) => {
                    match event.code {
                        KeyCode::Left | KeyCode::Char('a') => {
                            ctrl_tx.send(YardCtrl::CtrlSnake(id, Direction::L)).unwrap();
                        },
                        KeyCode::Right | KeyCode::Char('d') => {
                            ctrl_tx.send(YardCtrl::CtrlSnake(id, Direction::R)).unwrap();
                        },
                        KeyCode::Up | KeyCode::Char('w') => {
                            ctrl_tx.send(YardCtrl::CtrlSnake(id, Direction::U)).unwrap();
                        },
                        KeyCode::Down | KeyCode::Char('s') => {
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

/// checking if buffer is sended by the server, and print
pub fn polling_buf(id: u64, mut ui: render::TUIHelper, info_rx: Receiver<YardInfo>) {
    loop {
        thread::sleep(Duration::from_millis(10));
        match info_rx.try_recv() {
            Ok(info) => {
                match info {
                    YardInfo::RefreshScreen(buf) => {
                        ui.refresh_yard(buf).unwrap();
                    },
                    YardInfo::Board(s) => {
                        ui.print_info(&s).unwrap();
                    },
                    YardInfo::Failed(fid) => {
                        if fid == id {
                            ui.print_info(
                                    "Oops, try next time! press ESC to return to the menu."
                                ).unwrap();
                            return;
                        }
                    },
                    _ => {},
                };
            },
            Err(TryRecvError::Empty) => {},
            Err(TryRecvError::Disconnected) => { return; },
        }
    }
}

/// client main procedure
pub fn start_and_play(
        name: String,
        info_rx: Receiver<YardInfo>,
        ctrl_tx: Sender<YardCtrl>,
    ) {
    let ui = render::TUIHelper::new();
    let id: u64 = thread_rng().gen_range(u64::MIN..u64::MAX);
    ctrl_tx.send(YardCtrl::NewSnake(id, name)).unwrap();
    loop {
        match info_rx.recv().unwrap() {
            YardInfo::RegisteredSnake(rid, result) => {
                if rid == id && result {
                    break;
                }
            },
            _ => {},
        }
        println!("Log in failed, may due to congestion, please wait...");
    };
    let refresing_handle = thread::spawn(move || {
        polling_buf(id, ui, info_rx);
    });
    let keyboard_handle = thread::spawn(move || {
        polling_keyboard(id, ctrl_tx);
    });

    refresing_handle.join().unwrap();
    keyboard_handle.join().unwrap();
}