/// pub mod snakeux: user experience before and after actual game rendering

use std::io::{ stdin, stdout, Error, ErrorKind };

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

pub const MENU_HINT: &str = r#"
--------------------------------------------------------------------
                    Welcome To Socket Snake!
                    (1) Start singleplayer game
                    (2) Join a hosted game
                    (3) Host a game
                    (4) Exit
--------------------------------------------------------------------
Please type in your option:
"#;

pub const CHOICE_RANGE: std::ops::Range::<u8> = 1..5;

pub enum UsersIdea {
    Singleplayer,
    JoinGame(String),
    HostGame,
    ExitGame,
}

/// show the menu, and returns the user's idea
pub fn show_main_menu() -> Result<UsersIdea> {
    stdout()
        .execute(Clear(ClearType::All))?
        .execute(cursor::MoveTo(0, 0))?
        .execute(Print(TITLE))?
        .execute(Print(MENU_HINT))?;
    let mut line = String::new();
    stdin().read_line(&mut line).unwrap();
    let choice = loop {
        match line.trim().parse::<u8>() {
            Ok(num) if CHOICE_RANGE.contains(&num)
                => { break num; },
            _ => {
                println!("Please type in a number:");
                stdin().read_line(&mut line).unwrap();
            }
        }
    };
    match choice {
        1 => Ok(UsersIdea::Singleplayer),
        2 => Ok(UsersIdea::JoinGame("String".to_string())),
        3 => Ok(UsersIdea::HostGame),
        4 => Ok(UsersIdea::ExitGame),
        _ => Err(Error::new(ErrorKind::Other, "Choice out of range")),
    }
}