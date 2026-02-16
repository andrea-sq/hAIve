use crate::board::Rules;
use minimax::Game;
use rand::prelude::*;
use rand::rngs::SmallRng;

pub struct BiasedRollouts {}

impl minimax::RolloutPolicy for BiasedRollouts {
    type G = Rules;

    fn random_move(
        &self,
        board: &mut <Self::G as Game>::S,
        turns: &mut Vec<<Self::G as Game>::M>,
        rng: &mut SmallRng,
    ) -> <Self::G as Game>::M {
        // TODO: Lazily generate moves
        Rules::generate_moves(board, turns);
        let n = turns.len();
        turns.rotate_left(rng.random_range(0..n));
        for &turn in turns.iter() {
            board.apply(turn);
            if Rules::get_winner(board) == Some(minimax::Winner::PlayerJustMoved) {
                board.undo(turn);
                return turn;
            }
            board.undo(turn);
        }
        turns[0]
    }
}

unsafe impl Send for BiasedRollouts {}
