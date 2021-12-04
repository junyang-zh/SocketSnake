/// pub mod snakeux: user experience before and after actual game rendering

use crate::multiplayer;

use std::io::{ stdin, stdout, Error, ErrorKind };
use std::net::Ipv4Addr;

use rand::{ thread_rng, Rng };
use rand::prelude::SliceRandom;
pub use crossterm::{
    ExecutableCommand, QueueableCommand, Result,
    terminal::{Clear, ClearType},
    cursor,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};

pub const TITLE: &str = r#"
 ____   __    ___  __ _  ____  ____    ____  __ _   __   __ _  ____ 
/ ___) /  \  / __)(  / )(  __)(_  _)  / ___)(  ( \ / _\ (  / )(  __)
\___ \(  O )( (__  )  (  ) _)   )(    \___ \/    //    \ )  (  ) _) 
(____/ \__/  \___)(__\_)(____) (__)   (____/\_)__)\_/\_/(__\_)(____)
"#;

pub const GREETING: &str = r#"
--------------------------------------------------------------------
                    Hi! "#;

pub const MENU_HINT: &str = r#"
                    Welcome To Socket Snake!
                    (1) Start singleplayer game
                    (2) Join a hosted game
                    (3) Host a game
                    (4) Change your name
                    (5) Exit
--------------------------------------------------------------------
Please type in your option:
"#;

pub const HOST_HINT: &str = r#"
--------------------------------------------------------------------
     Socket Snake host is running on LocalHost "#;
pub const SEPERATOR: &str = r#"
                   Press Ctrl+C to End serving
--------------------------------------------------------------------
"#;

pub const CHOICE_RANGE: std::ops::Range::<u8> = 1..6;
pub const DEFAULT_NAMES: [&str; 9] = [
        "Happy Pants",
        "Mighty_Lord_Cobra",
        "__ADMINISTRATOR",
        "lol",
        "nitrogen",
        "misery-overcoat",
        "I am Groot",
        "BATMAN",
        "Zhang Weizhan",
    ];

pub enum UsersIdea {
    Singleplayer,
    JoinGame(String),
    HostGame,
    ChangeName,
    ExitGame,
}

pub fn input_ip_addr_port() -> String {
    // ask for input again if does not apply
    loop {
        println!("Please input the host ip and port in the format of 114.51.41.91:9810");
        let mut line = String::new();
        stdin().read_line(&mut line).unwrap();
        match line.trim().split(':').collect::<Vec<&str>>()[..] {
            [s_addr, s_port] => {
                match s_addr.parse::<Ipv4Addr>() {
                    Ok(_addr) => {
                        match s_port.parse::<u16>() {
                            Ok(_port) => {
                                break line.trim().to_string(); // return
                            },
                            _ => {},
                        }
                    },
                    _ => {},
                }
            },
            _ => {},
        }
    }
}

/// random from default names
pub fn random_name() -> String {
    DEFAULT_NAMES.choose(&mut thread_rng()).unwrap().to_string()
}

/// show the menu, and returns the user's idea
pub fn show_main_menu(name: &mut String) -> Result<UsersIdea> {
    stdout()
        .execute(Clear(ClearType::All))?
        .execute(cursor::MoveTo(0, 0))?
        .execute(Print(TITLE))?
        .execute(Print(GREETING))?
        .execute(Print(&format!("{}", &name)))?
        .execute(Print(MENU_HINT))?
        .execute(cursor::Show).unwrap();
    let mut line = String::new();
    stdin().read_line(&mut line).unwrap();
    let choice = loop {
        match line.trim().parse::<u8>() {
            Ok(num) if CHOICE_RANGE.contains(&num)
                => { break num; },
            _ => {
                println!("Please type in a number:");
                line = String::new();
                stdin().read_line(&mut line).unwrap();
            }
        }
    };
    match choice {
        1 => {
            Ok(UsersIdea::Singleplayer)
        },
        2 => {
            let addr = input_ip_addr_port();
            Ok(UsersIdea::JoinGame(addr))
        },
        3 => {
            stdout()
                .execute(Clear(ClearType::All))?
                .execute(cursor::MoveTo(0, 0))?
                .execute(Print(TITLE))?
                .execute(Print(HOST_HINT))?
                .execute(Print(multiplayer::TCP_SERVER_PORT))?
                .execute(Print(SEPERATOR))?
                .execute(cursor::Show).unwrap();
            Ok(UsersIdea::HostGame)
        },
        4 => {
            println!("Please enter your name:");
            let mut line = String::new();
            stdin().read_line(&mut line).unwrap();
            *name = line.trim().to_string();
            Ok(UsersIdea::ChangeName)
        },
        5 => {
            Ok(UsersIdea::ExitGame)
        },
        _ => Err(Error::new(ErrorKind::Other, "Choice out of range")),
    }
}