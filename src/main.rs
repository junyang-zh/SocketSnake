pub mod yard;
pub mod render;
pub mod server;
pub mod client;
pub mod snakeux;
pub mod multiplayer;

fn main() {
    loop {
        let choice = snakeux::show_main_menu().unwrap();
        match choice {
            snakeux::UsersIdea::Singleplayer
                => { multiplayer::singleplayer_start(); },
            snakeux::UsersIdea::JoinGame(addr)
                => { multiplayer::client_start(addr); },
            snakeux::UsersIdea::HostGame
                => { multiplayer::server_start().unwrap(); },
            snakeux::UsersIdea::ExitGame
                => { break; }
        }
    }
}
