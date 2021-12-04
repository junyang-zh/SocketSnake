pub mod yard;
pub mod render;
pub mod server;
pub mod client;
pub mod snakeux;
pub mod multiplayer;

fn main() {
    let mut name = snakeux::random_name(); // will be used next time, be sure to clone
    loop {
        let choice = snakeux::show_main_menu(&mut name).unwrap();
        match choice {
            snakeux::UsersIdea::Singleplayer
                => { multiplayer::singleplayer_start(name.clone()); },
            snakeux::UsersIdea::JoinGame(addr)
                => { multiplayer::client_start(name.clone(), addr); },
            snakeux::UsersIdea::HostGame
                => { multiplayer::server_start().unwrap(); },
            snakeux::UsersIdea::ChangeName
                => {},
            snakeux::UsersIdea::ExitGame
                => { break; }
        }
    }
}
