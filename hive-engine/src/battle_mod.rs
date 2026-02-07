use crate::player::Player;
use crate::uhp_client::UhpPlayer;
use crate::{Board, PlayerConfig, Rules};
use minimax::Game;
use std::time::Duration;

fn exit(msg: String) -> ! {
    eprintln!("{msg}");
    std::process::exit(1)
}

fn face_off(
    game_type: &str,
    mut player1: Box<dyn Player>,
    mut player2: Box<dyn Player>,
) -> Option<String> {
    let mut b = Board::from_game_type(game_type).unwrap();
    player1.new_game(game_type);
    player2.new_game(game_type);
    let mut players = [player1, player2];
    let mut p = 0;
    loop {
        // b.println();
        println!("{} ({:?}) to move", players[p].name(), b.to_move());
        let m = players[p].generate_move();
        let mut moves = Vec::new();
        Rules::generate_moves(&b, &mut moves);
        if !moves.contains(&m) {
            println!(
                "{} played an illegal move: {}",
                players[p].name(),
                b.to_move_string(m)
            );
            println!("Game log: {}", b.game_log());
            return Some(players[1 - p].name());
        }
        b.apply(m);
        if let Some(winner) = Rules::get_winner(&b) {
            // b.println();
            println!("Game log: {}", b.game_log());
            return match winner {
                minimax::Winner::Draw => None,
                minimax::Winner::PlayerJustMoved => Some(players[p].name()),
                minimax::Winner::PlayerToMove => Some(players[1 - p].name()),
            };
        }
        players[p].play_move(m);
        p = 1 - p;
        players[p].play_move(m);
    }
}

fn get_player(name: &str, config: &PlayerConfig) -> Box<dyn Player> {
    match name {
        "ai" => config.new_player(),
        // Try to launch this as a UHP server
        _ => Box::new(UhpPlayer::new(name).unwrap()),
    }
}

pub fn play_game(
    config: PlayerConfig,
    game_type: &str,
    name1: &str,
    name2: &str,
    depth: Option<u8>,
    timeout: Option<String>,
) {
    let mut player1 = get_player(name1, &config);
    let mut player2 = get_player(name2, &config);
    if let Some(depth) = depth {
        player1.set_max_depth(depth);
        player2.set_max_depth(depth);
    } else if let Some(input) = timeout {
        let timeout = if input.ends_with('s') {
            input[..input.len() - 1]
                .parse::<u64>()
                .map(Duration::from_secs)
        } else if input.ends_with('m') {
            input[..input.len() - 1]
                .parse::<u64>()
                .map(|m| Duration::from_secs(m * 60))
        } else {
            exit("Could not parse --timeout (add units)".to_string());
        }
        .unwrap_or_else(|_| exit("Could not parse --timeout (add units)".to_string()));
        player1.set_timeout(timeout);
        player2.set_timeout(timeout);
    }
    match face_off(game_type, player1, player2) {
        None => println!("Game over: draw."),
        Some(name) => println!("Game over: {name} won."),
    }
}
