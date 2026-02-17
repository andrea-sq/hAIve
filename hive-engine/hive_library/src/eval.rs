use crate::board::{Board, Color, Rules, Turn, START_HEX};
use crate::bug::Bug;
use crate::hex_grid::Hex;
use hexx::EdgeDirection;
use minimax::{Evaluation, Evaluator, Game};

pub struct DumbEvaluator;

impl Evaluator for DumbEvaluator {
    type G = Rules;

    fn evaluate(&self, _s: &<Self::G as Game>::S) -> Evaluation {
        0
    }
}

#[derive(Copy, Clone, Debug)]
pub struct BasicEvaluator {
    aggression: Evaluation,
    queen_liberty_factor: Evaluation,
    movable_queen_value: Evaluation,
    movable_bug_factor: Evaluation,
    unplayed_bug_factor: Evaluation,
    pillbug_defense_factor: Evaluation,
}

impl BasicEvaluator {
    pub(crate) fn new(aggression: u8) -> Self {
        let aggression = aggression.clamp(1, 5) as Evaluation;
        Self {
            aggression,
            queen_liberty_factor: aggression * 10,
            movable_queen_value: aggression * 4,
            movable_bug_factor: 2,
            unplayed_bug_factor: 1,
            pillbug_defense_factor: aggression * 40,
        }
    }
    pub(crate) fn aggression(&self) -> u8 {
        self.aggression as u8
    }
    fn value(&self, bug: Bug) -> Evaluation {
        match bug {
            Bug::Queen => self.movable_queen_value,
            Bug::Mosquito => 8,
            Bug::Ant => 7,
            Bug::Beetle => 6,
            Bug::Ladybug => 6,
            Bug::Pillbug => 5,
            Bug::Grasshopper => 3,
            Bug::Spider => 2,
        }
    }
}

impl Default for BasicEvaluator {
    fn default() -> Self {
        Self::new(3)
    }
}

fn count_liberties(board: &Board, origin: Hex, hex: Hex) -> Evaluation {
    board
        .adjacent(hex)
        .into_iter()
        .filter(|&adj| adj == origin || !board.occupied(adj))
        .count() as Evaluation
}

fn placeable(board: &Board, hex: Hex, color: Color) -> bool {
    !board
        .adjacent(hex)
        .any(|adj| board.occupied(adj) && board.node(adj).get_color().unwrap() != color)
}

#[test]
fn test_placeable() {
    let b = Board::from_game_string("Base;;;wA1;bA1 wA1-;wA2 /wA1").unwrap();

    assert!(!placeable(
        &b,
        b.neighbor(START_HEX, EdgeDirection::POINTY_SOUTH_EAST),
        Color::White
    ));
    assert!(!placeable(
        &b,
        b.neighbor(START_HEX, EdgeDirection::POINTY_NORTH_EAST),
        Color::White
    ));
    assert!(placeable(
        &b,
        b.neighbor(START_HEX, EdgeDirection::POINTY_NORTH_WEST),
        Color::White
    ));
    assert!(!placeable(
        &b,
        b.neighbor(START_HEX, EdgeDirection::POINTY_SOUTH_EAST),
        Color::Black
    ));
    assert!(!placeable(
        &b,
        b.neighbor(START_HEX, EdgeDirection::POINTY_NORTH_EAST),
        Color::Black
    ));
    assert!(!placeable(
        &b,
        b.neighbor(START_HEX, EdgeDirection::POINTY_NORTH_WEST),
        Color::Black
    ));
}

impl Evaluator for BasicEvaluator {
    type G = Rules;

    fn evaluate(&self, board: &<Self::G as Game>::S) -> Evaluation {
        let mut buf = [Hex::ZERO; 6];
        let mut immovable = board.find_cut_vertices();

        let mut score = 0;
        let mut pillbug_defense = [false; 2];
        let mut queen_score = [0; 2];

        let remaining = board.get_remaining();
        let opp_remaining = board.get_opponent_remaining();

        score += Bug::iter_all()
            .map(|bug| {
                (remaining[bug as usize] as Evaluation - opp_remaining[bug as usize] as Evaluation)
                    * self.value(bug)
            })
            .sum::<Evaluation>()
            * self.unplayed_bug_factor;

        for &hex in board.occupied_hexes[0]
            .iter()
            .chain(board.occupied_hexes[1].iter())
        {
            let node = board.node(hex);
            let mut bug_score = self.value(node.get_bug().unwrap());
            let mut pillbug_powers = node.get_bug().unwrap() == Bug::Pillbug;
            let mut crawler = node.get_bug().unwrap().crawler();

            if node.get_bug().unwrap() == Bug::Mosquito {
                bug_score = 0;
                crawler = true;
                if node.is_stacked() {
                    bug_score = self.value(Bug::Beetle);
                } else {
                    let mut adjacent_number = 0;
                    board
                        .adjacent(hex)
                        .filter(|&adj| board.occupied(adj))
                        .map(|adj| board.node(adj).get_bug().unwrap())
                        .for_each(|bug| {
                            if bug == Bug::Queen || bug == Bug::Mosquito {
                                adjacent_number += 1;
                                bug_score = bug_score.max(self.value(bug));
                            }
                            if bug == Bug::Pillbug {
                                pillbug_powers = true;
                            }
                            if !bug.crawler() {
                                crawler = false;
                            }
                        });
                    bug_score += adjacent_number as Evaluation;
                }
            }

            if crawler
                && board
                    .slideable_adjacent(&mut buf, hex, hex)
                    .next()
                    .is_none()
            {
                immovable.insert(board.find_id(hex));
            }
            if node.is_stacked() {
                bug_score *= 2;
            }
            let friendly_queen = board.queens[node.get_color().unwrap() as usize];

            // TODO: Transpose this out of the loop
            if board.adjacent(friendly_queen).any(|adj| adj == hex) {
                if immovable.contains(board.find_id(hex)) && !node.is_stacked() {
                    queen_score[node.get_color().unwrap() as usize] -= self.queen_liberty_factor;
                } else {
                    queen_score[node.get_color().unwrap() as usize] -=
                        self.queen_liberty_factor / 2;
                }
                if pillbug_powers && board.node(friendly_queen).get_tile_height() == 1 {
                    let best_escape = board
                        .adjacent(hex)
                        .into_iter()
                        .map(|lib| {
                            if board.occupied(lib) {
                                0
                            } else {
                                count_liberties(board, friendly_queen, lib)
                            }
                        })
                        .max()
                        .unwrap_or(0);
                    if best_escape > 2 {
                        pillbug_defense[node.get_color().unwrap() as usize] = true;
                    }
                }
            }
            let enemy_queen = board.queens[node.get_color().unwrap().other() as usize];

            if board.adjacent(enemy_queen).any(|adj| adj == hex) {
                bug_score = 0;
                queen_score[node.get_color().unwrap().other() as usize] -=
                    self.queen_liberty_factor * 12 / 10;
                if pillbug_powers {
                    let best_unescape = board
                        .adjacent(hex)
                        .map(|lib| {
                            if board.occupied(lib) {
                                6
                            } else {
                                count_liberties(board, enemy_queen, lib)
                            }
                        })
                        .min()
                        .unwrap_or(6);
                    if best_unescape < 3 {
                        queen_score[node.get_color().unwrap().other() as usize] =
                            -self.queen_liberty_factor;
                    }
                }
            }
            if !node.is_stacked() && immovable.contains(board.find_id(hex)) {
                continue;
            }
            bug_score *= self.movable_bug_factor;
            if node.get_color().unwrap() != board.to_move() {
                bug_score = -bug_score;
                if self.aggression == 1 {
                    bug_score *= 2;
                } else if self.aggression == 2 {
                    bug_score = bug_score * 3 / 2;
                }
            }
            score += bug_score;
        }
        let mut pillbug_defense_score = self.pillbug_defense_factor
            * (pillbug_defense[board.to_move() as usize] as Evaluation
                - pillbug_defense[board.to_move().other() as usize] as Evaluation);
        pillbug_defense = [false; 2];
        for &color in &[Color::White, Color::Black] {
            if board.node(board.queens[color as usize]).get_tile_height() == 1
                && board.remaining[color as usize][Bug::Pillbug as usize] > 0
                && board
                    .adjacent(board.queens[color as usize])
                    .any(|lib| placeable(board, lib, color))
            {
                pillbug_defense[color as usize] = true;
            }
        }
        pillbug_defense_score += self.pillbug_defense_factor / 2
            * (pillbug_defense[board.to_move() as usize] as Evaluation
                - pillbug_defense[board.to_move().other() as usize] as Evaluation);
        let queen_score =
            queen_score[board.to_move() as usize] - queen_score[board.to_move().other() as usize];
        queen_score + pillbug_defense_score + score
    }

    fn generate_noisy_moves(
        &self,
        board: &<Self::G as Game>::S,
        moves: &mut Vec<<Self::G as Game>::M>,
    ) {
        if board.turn_history.len() < 4 || board.get_remaining()[Bug::Queen as usize] == 0 {
            return;
        }
        let enemy_last_move = board.turn_history[board.turn_history.len() - 1];
        let my_last_move = board.turn_history[board.turn_history.len() - 2];

        if let Turn::Place(hex, _) = my_last_move
            && !board
                .adjacent(board.queens[board.to_move().other() as usize])
                .any(|adj| adj == hex)
        {
            board.generate_movements(moves);
            moves.retain(|m| {
                if let Turn::Move(start, _) = *m {
                    start == hex
                } else {
                    false
                }
            });
            return;
        }
        if let Turn::Place(hex, _) = enemy_last_move
            && !board
                .adjacent(board.queens[board.to_move() as usize])
                .any(|adj| adj == hex)
        {
            board.generate_movements(moves);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::GRID_RADIUS;
    use hexx::{HexOrientation, OffsetHexMode};

    fn loc_to_hex(loc: (i8, i8)) -> Hex {
        Hex(hexx::Hex::from_offset_coordinates(
            [loc.0 as i32, loc.1 as i32],
            OffsetHexMode::Even,
            HexOrientation::Pointy,
        )
        .to_hexmod_coordinates(GRID_RADIUS as u32) as u16)
    }

    #[test]
    fn test_minimax() {
        use minimax::{Negamax, Strategy};

        // Find the winning move.
        // ．．．Q S ．．
        //．．A A Q ．．
        // ．．．G B
        let mut board = Board::default();
        board.apply(Turn::Place(loc_to_hex((0, 0)), Bug::Queen));
        board.apply(Turn::Place(loc_to_hex((1, 0)), Bug::Spider));
        board.apply(Turn::Place(loc_to_hex((-1, 1)), Bug::Ant));
        board.apply(Turn::Place(loc_to_hex((0, 1)), Bug::Ant));
        board.apply(Turn::Place(loc_to_hex((0, 2)), Bug::Grasshopper));
        board.apply(Turn::Place(loc_to_hex((1, 1)), Bug::Queen));
        board.apply(Turn::Place(loc_to_hex((1, 2)), Bug::Beetle));
        board.apply(Turn::Pass);
        for depth in 1..3 {
            let mut strategy = Negamax::new(DumbEvaluator {}, depth);
            let m = strategy.choose_move(&board);
            assert_eq!(Some(Turn::Move(loc_to_hex((-1, 1)), loc_to_hex((2, 1)))), m);

            let mut strategy = Negamax::new(BasicEvaluator::default(), depth);
            let m = strategy.choose_move(&board);
            assert_eq!(Some(Turn::Move(loc_to_hex((-1, 1)), loc_to_hex((2, 1)))), m);
        }

        // Find queen escape.
        //．．B Q Q ．
        // ．．G S ．
        let mut board = Board::default();
        board.apply(Turn::Place(loc_to_hex((0, 0)), Bug::Queen));
        board.apply(Turn::Place(loc_to_hex((1, 0)), Bug::Queen));
        board.apply(Turn::Place(loc_to_hex((1, 1)), Bug::Spider));
        board.apply(Turn::Place(loc_to_hex((0, 1)), Bug::Grasshopper));
        board.apply(Turn::Place(loc_to_hex((-1, 0)), Bug::Beetle));
        board.apply(Turn::Pass);
        for depth in 1..3 {
            let mut strategy = Negamax::new(BasicEvaluator::default(), depth);
            let m = strategy.choose_move(&board);
            assert_eq!(Some(Turn::Move(loc_to_hex((0, 0)), loc_to_hex((1, -1)))), m);
        }
    }
}
