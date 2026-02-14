use crate::player::{Player, PlayerStrategy};
use crate::{BasicEvaluator, Board, PlayerConfig, Rules};
use minimax::{Evaluator, Game};
use rand::{rng, Rng};
use serde::{Deserialize, Serialize};
use std::io;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BoardItem {
    white_features_indices: Vec<usize>,
    black_features_indices: Vec<usize>,
    stm: bool,
    score: f64,
}

pub fn generate_games<W: io::Write>(
    games: usize,
    min_after_rounds: usize,
    max_after_rounds: usize,
    mut wrt: W,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut current_games = 0;
    'outer_loop: while current_games < games {
        let mut player1_config = PlayerConfig::new();
        let mut player2_config = PlayerConfig::new();
        player1_config.strategy = PlayerStrategy::Random;
        player2_config.strategy = PlayerStrategy::Random;
        let mut player1 = player1_config.new_player();
        let mut player2 = player2_config.new_player();
        let game_type = "Base+MLP";
        let mut b = Board::from_game_type(game_type).unwrap();
        player1.new_game(game_type);
        player2.new_game(game_type);
        let mut players = [player1, player2];
        let mut p = 0;

        let after_rounds = rng().random_range(min_after_rounds..max_after_rounds);

        for _ in 0..after_rounds {
            let m = players[p].generate_move();
            let mut moves = Vec::new();
            Rules::generate_moves(&b, &mut moves);
            b.apply(m);
            if let Some(_) = Rules::get_winner(&b) {
                continue 'outer_loop;
            }
            players[p].play_move(m);
            p = 1 - p;
            players[p].play_move(m);
        }

        let basic_evaluator = BasicEvaluator::default();

        let mut white_features_indices_vec = b.nnue_board.get_white_feature_indices();
        let white_features_indices = white_features_indices_vec.try_into().unwrap();

        let mut black_features_indices_vec = b.nnue_board.get_black_feature_indices();
        let black_features_indices = black_features_indices_vec.try_into().unwrap();

        let stm = p == 0;
        let score = basic_evaluator.evaluate(&b) as f64;

        let board_item = BoardItem {
            white_features_indices,
            black_features_indices,
            stm,
            score,
        };

        let string = serde_json::to_string(&board_item)?;
        writeln!(&mut wrt, "{}", string)?;

        current_games += 1;
    }
    Ok(())
}
