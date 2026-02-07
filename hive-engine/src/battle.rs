use hive_engine::{configure_player, play_game};
use std::ffi::OsString;

fn main() {
    let (config, args) = configure_player().unwrap();
    let mut args =
        pico_args::Arguments::from_vec(args.iter().map(|s| s.into()).collect::<Vec<OsString>>());
    let game_type = args
        .opt_value_from_str("--game-type")
        .unwrap()
        .unwrap_or_else(|| "Base+MLP".to_owned());
    let depth: Option<u8> = args.opt_value_from_str("--depth").unwrap();
    let timeout: Option<String> = args.opt_value_from_str("--timeout").unwrap();
    let args = args
        .finish()
        .into_iter()
        .map(|s| s.into_string().unwrap())
        .collect::<Vec<_>>();
    let player1 = args.get(1).map(|s| s.as_str()).unwrap_or("ai");
    let player2 = args.get(2).map(|s| s.as_str()).unwrap_or("ai");
    play_game(config, &game_type, player1, player2, depth, timeout);
}
