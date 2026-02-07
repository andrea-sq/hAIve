use crate::player::configure_player;
use crate::uhp_server::uhp_serve;

mod board;
mod bug;
mod eval;
mod mcts;
mod notation;
mod player;
mod random;
mod uhp_server;

fn main() {
    let (config, args) = configure_player().unwrap();
    match args.first().unwrap_or(&"uhp".to_owned()).as_ref() {
        "uhp" => {
            uhp_serve(config);
        }
        _ => panic!("Unrecognized command: {}", args.join(" ")),
    }
}
