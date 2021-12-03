/// pub mod client: have a fn that can be started as a thread
/// interact with the server and user

use crate::render;
use crate::server::{ YardCtrl, YardInfo };
use crate::yard::{ YardBuf, Direction };

use crossterm::event::{ poll, read, Event, KeyCode };

use std::thread;
use std::time::Duration;
use std::sync::mpsc::{ Sender, Receiver, TryRecvError };

/// polling keyboard strike and customize control
/// id: client id, ctrl_tx: control signal channel
pub fn polling_keyboard(id: u8, ctrl_tx: Sender<YardCtrl>) {
    loop {
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

/// checking if buffer is sended by the server, and print
pub fn polling_buf(mut ui: render::TUIHelper, buf_rx: Receiver<YardBuf>) {
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
}

/// client main procedure
pub fn start_and_play(
        buf_rx: Receiver<YardBuf>,
        info_rx: Receiver<YardInfo>,
        ctrl_tx: Sender<YardCtrl>,
    ) {
    let ui = render::TUIHelper::new();
    ctrl_tx.send(YardCtrl::NewSnake).unwrap();
    let id;
    match info_rx.recv().unwrap() {
        YardInfo::RegisteredSnake(given_id) => { id = given_id.unwrap(); },
        _ => { panic!("Room may full") },
    }
    let refresing_handle = thread::spawn(move || {
        polling_buf(ui, buf_rx);
    });
    let keyboard_handle = thread::spawn(move || {
        polling_keyboard(id, ctrl_tx);
    });

    refresing_handle.join().unwrap();
    keyboard_handle.join().unwrap();
}