/// pub mod yard:
///  - get commands from multiple users
///  - simulate the game
///  - produce the screen buffer

pub use crossterm::style::Color;
pub use crate::render::{
    HEAD_L, HEAD_R, HEAD_U, HEAD_D, BEAN, FENCE, EMPTY,
    TUIBlock,
};
pub use std::collections::VecDeque;

/// coordinate on the field as (row, column)
#[derive(Copy, Clone)]
pub struct Coord(usize, usize);

#[derive(Copy, Clone)]
pub enum Direction { L, R, U, D, }

impl Coord {
    pub fn move_toward(&self, d: Direction, bounds: Coord) -> Option<Coord> {
        match d {
            Direction::L if self.1 > 0
                => Some(Coord(self.0, self.1 - 1)),
            Direction::R if self.1 + 1 < bounds.1
                => Some(Coord(self.0, self.1 + 1)),
            Direction::U if self.0 > 0
                => Some(Coord(self.0 - 1, self.1)),
            Direction::D if self.0 + 1 < bounds.0
                => Some(Coord(self.0 + 1, self.1)),
            _ => None,
        }
    }
}

/// maximum players on the yard
pub const MAX_PLAYERS: u8 = 5;

/// abstract representitive of blocks managed by the server
/// user ids are assigned as u8 since there's at most 5 sessions
#[derive(Copy, Clone)]
pub enum YardBlockType {
    Empty,
    Bean,
    Body(u8),
    Head(u8, Direction),
}
use YardBlockType::{ Empty, Bean, Body, Head };

pub const PLAYER_COLOR_MAP: [Color; MAX_PLAYERS as usize]
    = [Color::DarkGrey, Color::DarkRed, Color::DarkBlue, Color::DarkMagenta, Color::DarkCyan];

pub fn get_tui_block(b: &YardBlockType) -> Option<TUIBlock> {
    match b {
        Empty
            => Some(TUIBlock {
                fg: Color::White,
                bg: Color::White,
                content: EMPTY,
            }),
        Bean
            => Some(TUIBlock {
                fg: Color::Yellow,
                bg: Color::Green,
                content: BEAN,
            }),
        Body(id) if *id < MAX_PLAYERS
            => Some(TUIBlock {
                fg: Color::White,
                bg: PLAYER_COLOR_MAP[*id as usize],
                content: EMPTY,
            }),
        Head(id, d) if *id < MAX_PLAYERS
            => Some(TUIBlock {
                fg: Color::White,
                bg: PLAYER_COLOR_MAP[*id as usize],
                content: match d {
                    Direction::L => HEAD_L,
                    Direction::R => HEAD_R,
                    Direction::U => HEAD_U,
                    Direction::D => HEAD_D,
                },
            }),
        _ => None,
    }
}

/// snakes which have a head and direction, the head is the front element
pub struct Snake(VecDeque<Coord>, Direction);

/// game simulator
pub struct YardSim {
    width: usize,
    height: usize,
    tick: u64,
    bean_count: usize,
    block_map: Vec<YardBlockType>,                  // without borders, thus with shape w * h
    snakes: [Option<Snake>; MAX_PLAYERS as usize],  // there can be player ids not registered
    score: [usize; MAX_PLAYERS as usize],
    failed: [bool; MAX_PLAYERS as usize],           // mark fail and clean up
    bonused: [usize; MAX_PLAYERS as usize],
}

/// the buffer that is sended to the clients
pub type YardBuf = Vec<TUIBlock>;
impl YardSim {
    pub fn new(width: usize, height: usize, bean_count: usize) -> YardSim {
        YardSim {
            width, height,
            tick: 0,
            bean_count,
            block_map: vec![Empty; width * height],
            /// [None; MAX_PLAYERS as usize] won't work well, so make it clumsy
            snakes: [None, None, None, None, None],
            score: [0; MAX_PLAYERS as usize],
            failed: [false; MAX_PLAYERS as usize],
            bonused: [0; MAX_PLAYERS as usize],
        }
    }

    /// generate a buffer for the client to print
    pub fn generate_buf(&self) -> YardBuf {
        let mut result_buf = YardBuf::new();
        for r in 0..self.height {
            for c in 0..self.width {
                result_buf.push(get_tui_block(&self.block_map[r * c]).unwrap());
            }
        }
        result_buf
    }
    
    pub fn control_snake(&mut self, id: u8, d: Direction) -> Option<()> {
        match &self.snakes[id as usize] {
            Some(s) => { s.1 = d; () },
            None => None,
        }
    }

    pub fn get_score_of(&self) {

    }

    /// simulate the game:
    ///  - update each snake's position by its direction
    ///  - decide if gets point or fails
    pub fn next_tick(&mut self) {
        for id in 0..MAX_PLAYERS {
            match &mut self.snakes[id as usize] {
                Some(s) => {
                    let new_head
                        = match s.0.front().unwrap().move_toward(s.1, Coord(self.width, self.height)) {
                            Some(pos) => pos,
                            None => {
                                self.failed[id as usize] = true;
                                continue;
                            },
                        };
                    s.0.push_front(new_head);
                    // going to one's retracting nail would fail, mark and cleanup strategy
                    match &self.block_map[new_head.0 * new_head.1] {
                        Empty => {              // go over and let tail retract
                            self.block_map[new_head.0 * new_head.1] = Head(id, s.1);
                            s.0.pop_back().unwrap();
                        },
                        Bean => {               // go over extending head
                            self.block_map[new_head.0 * new_head.1] = Head(id, s.1);
                            self.bonused[id as usize] += 1;
                        },
                        Body(id_at) => {
                            self.failed[id as usize] = true;                 // mark this snake fail
                            self.bonused[*id_at as usize] += self.score[id as usize]; // bonus the attacker
                            continue;
                        },
                        Head(id_at, _d) => {     // judge if head-to-head collision
                            if id < *id_at {
                                self.failed[id as usize] = true;
                                self.bonused[*id_at as usize] += self.score[id as usize];
                                continue;
                            } else {
                                self.failed[id as usize] = true;
                                self.failed[*id_at as usize] = true;
                                continue;
                            }
                        },
                    }
                },
                None => continue,
            }
        }
        self.tick += 1;
    }

    /// cleanup after it ticks
    pub fn cleanup(&mut self) {
        for id in 0..(MAX_PLAYERS as usize) {
            if self.bonused[id] > 0 {
                self.score[id] += self.bonused[id];
                self.bonused[id] = 0;
            }
            if self.failed[id] {
                // clean up mess
                for each_pos in &self.snakes[id].as_ref().unwrap().0 {
                    self.block_map[each_pos.0 * each_pos.1] = Empty;
                }
                // unregister a snake
                self.snakes[id] = None;
                self.failed[id] = false;
            }
        }
    }
}