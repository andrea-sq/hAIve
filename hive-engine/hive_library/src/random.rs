use minimax::{Game, Strategy};
use rand::prelude::*;
use std::marker::PhantomData;

pub struct Random<G: Game> {
    rng: SmallRng,
    game_type: PhantomData<G>,
}

impl<G: Game> Random<G> {
    pub fn new(rng: SmallRng) -> Self {
        Self {
            rng,
            game_type: PhantomData,
        }
    }
}

impl<G: Game> Default for Random<G> {
    fn default() -> Self {
        Self::new(SmallRng::from_os_rng())
    }
}

impl<G: Game> Strategy<G> for Random<G>
where
    G::M: Copy,
{
    fn choose_move(&mut self, s: &G::S) -> Option<G::M> {
        let mut moves = Vec::new();
        G::generate_moves(s, &mut moves);
        moves.choose(&mut self.rng).copied()
    }
}
