//! # socket_snake
//! This is firstly a software laboratory for the XJTU course computer network, then implemented multiplayer snake game, with Rust.
//! Main contibutors:
//!  - [James-Hen](https://github.com/James-Hen)
//!  - [yanxwb](https://github.com/yanxwb)

/// Snake game rule implementation, simulate step by step in ticks
pub mod yard;
/// Helper module that defined some ui rendering components and utilities
pub mod render;
/// Game simulation thread implementation
pub mod server;
/// User interface, game control threads implementation
pub mod client;
/// Defines the user interaction that improves user's experience
pub mod snakeux;
/// Server and client wrappers to introduce sockets and channels
pub mod multiplayer;
/// Helper module that defined some network transmitting components and utilities
#[macro_use]
pub mod transmit;

fn main() {
    let mut name = snakeux::random_name(); // will be used next time, be sure to clone
    loop {
        let choice = snakeux::show_main_menu(&mut name).unwrap();
        match choice {
            snakeux::UsersIdea::Singleplayer
                => { multiplayer::singleplayer_start(name.clone()); },
            snakeux::UsersIdea::JoinGame(addr)
                => { multiplayer::client_start(name.clone(), addr); },
            snakeux::UsersIdea::HostGame(addr)
                => { multiplayer::server_start(addr).unwrap(); },
            snakeux::UsersIdea::ChangeName
                => {},
            snakeux::UsersIdea::ExitGame
                => { break; }
        }
    }
}
