/// pub mod yard:
///  - get commands from multiple users
///  - simulate the game
///  - produce the screen buffer

pub use crate::render::{
    HEAD_L, HEAD_R, HEAD_U, HEAD_D, BEAN, FENCE, EMPTY,
    TUIBlock, YardBuf,
};

pub use std::collections::VecDeque;

pub use crossterm::style::Color;
use serde::{Deserialize, Serialize};
use rand::prelude::*;

/// coordinate on the field as (row, column)
#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Coord(usize, usize);

/// left, right, up, down
#[derive(Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum Direction { L, R, U, D, }
use Direction::{ L, R, U, D, };

impl Direction {
    /// find a next random value for Direction, can't turn back
    pub fn next_random(self) -> Self {
        match self {
            L => [R, U, D],
            R => [L, U, D],
            U => [L, R, D],
            D => [L, R, U],
        }
        .choose(&mut thread_rng())
        .copied().unwrap()
    }
    /// returns the opposite direction, useful when judging valid moves
    pub fn opposite(&self) -> Direction {
        match *self {
            L => R, R => L, U => D, D => U,
        }
    }
}

impl Coord {
    /// return the moved Coord and judge if within the (0,0)-bound rectangle
    /// note: (column, row)
    pub fn move_toward(&self, d: Direction, bounds: Coord) -> Option<Coord> {
        match d {
            L if self.1 > 0
                => Some(Coord(self.0, self.1 - 1)),
            R if self.1 + 1 < bounds.1
                => Some(Coord(self.0, self.1 + 1)),
            U if self.0 > 0
                => Some(Coord(self.0 - 1, self.1)),
            D if self.0 + 1 < bounds.0
                => Some(Coord(self.0 + 1, self.1)),
            _ => None,
        }
    }
    /// call `bound.rand_inside()`, return a Coord inside the (0,0)-bound rectangle
    pub fn rand_inside(&self) -> Coord {
        let mut rng = thread_rng();
        Coord(rng.gen_range(0..self.0), rng.gen_range(0..self.1))
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

/// snakes which have a head and direction, the head is the front element
pub struct Snake(VecDeque<Coord>, Direction);

/// game simulator
pub struct YardSim {
    // configurations
    width: usize,
    height: usize,
    bean_count: usize,
    init_snake_len: usize,
    // running status
    tick: u64,
    beans_left: usize,
    block_map: Vec<Vec<YardBlockType>>,             // without borders, thus with shape w * h
    snakes: [Option<Snake>; MAX_PLAYERS as usize],  // there can be player ids not registered
    stall_protect: [u64; MAX_PLAYERS as usize],     // newborns shall have some ticks to stall
    score: [usize; MAX_PLAYERS as usize],
    failed: [bool; MAX_PLAYERS as usize],           // mark fail and clean up
    bonused: [usize; MAX_PLAYERS as usize],
}

impl YardSim {
    /// create a simulator
    ///  - `width`: the columns count
    ///  - `height`: the rows count
    ///  - `bean_count`: the initial bean count, and it will hold forever
    ///  - `init_snake_len`: the length of the snake, don't make it large, otherwise hurts perf
    pub fn new(width: usize, height: usize, bean_count: usize, init_snake_len: usize) -> YardSim {
        let mut block_map = Vec::<Vec<YardBlockType>>::new();
        for _row in 0..height {
            block_map.push(vec![Empty; width]);
        }
        let mut y = YardSim {
                width, height, bean_count, init_snake_len,
                tick: 0,
                beans_left: 0,
                block_map,
                /// [None; MAX_PLAYERS as usize] won't work well, so make it clumsy
                snakes: [None, None, None, None, None],
                stall_protect: [0; MAX_PLAYERS as usize],
                score: [0; MAX_PLAYERS as usize],
                failed: [false; MAX_PLAYERS as usize],
                bonused: [0; MAX_PLAYERS as usize],
            };
        y.fill_beans(); // tries to generate beans
        y
    }

    /// generate a buffer for the client to print
    pub fn generate_buf(&self) -> YardBuf {
        let mut result_buf = YardBuf::new();
        for r in 0..self.height {
            result_buf.push(Vec::<TUIBlock>::new());
            for c in 0..self.width {
                let block = match &self.block_map[r][c] {
                    Empty
                        => Some(TUIBlock {
                            fg: Color::White,
                            bg: Color::White,
                            content: EMPTY.to_string(),
                        }),
                    Bean
                        => Some(TUIBlock {
                            fg: Color::Yellow,
                            bg: Color::Green,
                            content: BEAN.to_string(),
                        }),
                    Body(id) if *id < MAX_PLAYERS
                        => Some(TUIBlock {
                            fg: Color::White,
                            bg: if self.stall_protect[*id as usize] & 1 > 0 { Color::White }
                                else { PLAYER_COLOR_MAP[*id as usize] },
                            content: EMPTY.to_string(),
                        }),
                    Head(id, d) if *id < MAX_PLAYERS
                        => Some(TUIBlock {
                            fg: Color::White,
                            bg: if self.stall_protect[*id as usize] & 1 > 0 { Color::White }
                                else { PLAYER_COLOR_MAP[*id as usize] },
                            content: match d {
                                Direction::L => HEAD_L.to_string(),
                                Direction::R => HEAD_R.to_string(),
                                Direction::U => HEAD_U.to_string(),
                                Direction::D => HEAD_D.to_string(),
                            },
                        }),
                    _ => None,
                };
                result_buf[r].push(block.unwrap());
            }
        }
        result_buf
    }

    /// tries hard to create a snake on the field, return a id
    pub fn init_snake(&mut self) -> Option<u8> {
        // find a id to assign
        let mut i: u8 = 0;
        let id = loop {
                if i >= MAX_PLAYERS {
                    break None;
                }
                match &self.snakes[i as usize] {
                    Some(_s) => {},
                    None => break Some(i),
                }
                i += 1;
            };
        if id == None {
            return id;
        }
        // generate snake
        let bound = Coord(self.height, self.width);
        let mut segment: VecDeque<Coord>;
        let mut d;
        'choose_segment:
        loop {
            let mut tail = bound.rand_inside();
            d = [L, R, U, D].choose(&mut thread_rng()).copied().unwrap();
            segment = VecDeque::<Coord>::new();
            segment.push_front(tail);
            for _i in 1..self.init_snake_len {
                match tail.move_toward(d, bound) {
                    Some(c) => {
                        segment.push_front(c);
                        tail = c;
                    },
                    None => {
                        continue 'choose_segment;
                    },
                }
            }
            // judge if intersect with others
            for c in &segment {
                match self.block_map[c.0][c.1] {
                    Empty => continue,
                    _ => continue 'choose_segment,
                }
            }
            break; // finished nicely
        }
        // register snake
        let head = segment.front().unwrap();
        self.block_map[head.0][head.1] = Head(id.unwrap(), d);
        let mut iter = segment.iter();
        iter.next();
        for c in iter {
            self.block_map[c.0][c.1] = Body(id.unwrap());
        }
        self.snakes[id.unwrap() as usize] = Some(Snake(segment, d));
        self.stall_protect[id.unwrap() as usize] = 10; // set protection to 10 ticks
        id
    }

    pub fn control_snake(&mut self, id: u8, d: Direction) -> Option<()> {
        match &mut self.snakes[id as usize] {
            Some(s) => {
                let head = s.0.front().unwrap();
                // can't turn back, regarding the state before ticking
                match self.block_map[head.0][head.1] {
                    Head(bid, cur_dir) => {
                        if bid == id && d != cur_dir.opposite() {
                            s.1 = d;
                        }
                    },
                    _ => {},
                };
                Some(())
            },
            None => None,
        }
    }

    pub fn get_score_of(&self, id: u8) -> usize {
        self.score[id as usize]
    }

    /// produce new beans on the ground till satisfied
    pub fn fill_beans(&mut self) {
        while self.beans_left < self.bean_count {
            let bound = Coord(self.height, self.width);
            let loc = loop {
                let c = bound.rand_inside();
                match &self.block_map[c.0][c.1] {
                    Empty => { break c; },
                    _ => {},
                };
            };
            self.block_map[loc.0][loc.1] = Bean;
            self.beans_left += 1;
        }
    }

    /// clean up failed snakes, please do after ticks
    pub fn cleanup(&mut self) {
        for id in 0..(MAX_PLAYERS as usize) {
            if self.bonused[id] > 0 {
                self.score[id] += self.bonused[id];
                self.bonused[id] = 0;
            }
            if self.failed[id] {
                // clean up mess
                for each_pos in &self.snakes[id].as_ref().unwrap().0 {
                    self.block_map[each_pos.0][each_pos.1] = Empty;
                }
                // unregister a snake
                self.snakes[id] = None;
                self.failed[id] = false;
            }
        }
    }
    
    /// simulate the game:
    ///  - update each snake's position by its direction
    ///  - decide if gets point or fails
    pub fn next_tick(&mut self) {
        for id in 0..MAX_PLAYERS {
            // handle newborn protection
            if self.stall_protect[id as usize] > 0 {
                self.stall_protect[id as usize] -= 1;
                continue;
            }
            match &mut self.snakes[id as usize] {
                Some(s) => {
                    let head = s.0.front().unwrap();
                    let new_head
                        = match head.move_toward(s.1, Coord(self.height, self.width)) {
                            Some(pos) => pos,
                            None => {
                                self.failed[id as usize] = true;
                                continue;
                            },
                        };
                    self.block_map[head.0][head.1] = Body(id);
                    s.0.push_front(new_head);
                    // going to one's retracting nail would fail, mark and cleanup strategy
                    match &self.block_map[new_head.0][new_head.1] {
                        Empty => {              // go over and let tail retract
                            self.block_map[new_head.0][new_head.1] = Head(id, s.1);
                            let tail = s.0.pop_back().unwrap();
                            self.block_map[tail.0][tail.1] = Empty;
                        },
                        Bean => {               // go over extending head
                            self.block_map[new_head.0][new_head.1] = Head(id, s.1);
                            self.bonused[id as usize] += 1;
                            self.beans_left -= 1;
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
        self.cleanup();
        self.fill_beans();
    }
}