use crate::board::GRID_SIZE;
use crate::bug::Bug;
use crate::{Color, Node};
use hexx::{EdgeDirection, Hex};
use std::char::MAX;

pub const MAX_ACTIVE_FEATURES: usize = 28 + 6;
pub const MAX_BOARD_SIZE: usize = 3 * 14 + 3 * 14 * 14 + 1;
const EDGES_NNUE_BOARD: usize = MAX_BOARD_SIZE * 28;
const HEIGHTS_NNUE_BOARD: usize = 6 * 6;
pub const NNUE_BOARD: usize = EDGES_NNUE_BOARD + HEIGHTS_NNUE_BOARD;

#[derive(Debug, Clone)]
pub struct NNUEBoard {
    board: [bool; NNUE_BOARD],
}

impl Default for NNUEBoard {
    fn default() -> Self {
        Self::new()
    }
}

impl NNUEBoard {
    pub fn new() -> Self {
        Self {
            board: [false; NNUE_BOARD],
        }
    }

    pub fn get_white_feature_indices(&self) -> Vec<usize> {
        self.board
            .iter()
            .enumerate()
            .filter_map(|(i, b)| if *b { Some(i) } else { None })
            .collect()
    }

    pub fn get_black_feature_indices(&self) -> Vec<usize> {
        let half_edges = EDGES_NNUE_BOARD / 2;
        let half_heights = HEIGHTS_NNUE_BOARD / 2;

        self.board[half_edges..EDGES_NNUE_BOARD]
            .iter()
            .chain(self.board[0..half_edges].iter())
            .chain(self.board[(EDGES_NNUE_BOARD + half_heights)..NNUE_BOARD].iter())
            .chain(self.board[EDGES_NNUE_BOARD..(EDGES_NNUE_BOARD + half_heights)].iter())
            .enumerate()
            .filter_map(|(i, b)| if *b { Some(i) } else { None })
            .collect()
    }

    fn get_piece_number(node: Node) -> usize {
        let bug = node.get_bug().unwrap();
        let bug_num = node.get_bug_num();
        let color = node.get_color().unwrap();

        match (bug, bug_num, color) {
            //WHITE
            (Bug::Queen, 1, Color::White) => 0,

            (Bug::Grasshopper, 1, Color::White) => 1,
            (Bug::Grasshopper, 2, Color::White) => 2,
            (Bug::Grasshopper, 3, Color::White) => 3,

            (Bug::Spider, 1, Color::White) => 4,
            (Bug::Spider, 2, Color::White) => 5,

            (Bug::Ant, 1, Color::White) => 6,
            (Bug::Ant, 2, Color::White) => 7,
            (Bug::Ant, 3, Color::White) => 8,

            (Bug::Beetle, 1, Color::White) => 9,
            (Bug::Beetle, 2, Color::White) => 10,

            (Bug::Mosquito, 1, Color::White) => 11,

            (Bug::Ladybug, 1, Color::White) => 12,

            (Bug::Pillbug, 1, Color::White) => 13,
            //BLACK
            (Bug::Queen, 1, Color::Black) => 14,

            (Bug::Grasshopper, 1, Color::Black) => 15,
            (Bug::Grasshopper, 2, Color::Black) => 16,
            (Bug::Grasshopper, 3, Color::Black) => 17,

            (Bug::Spider, 1, Color::Black) => 18,
            (Bug::Spider, 2, Color::Black) => 19,

            (Bug::Ant, 1, Color::Black) => 20,
            (Bug::Ant, 2, Color::Black) => 21,
            (Bug::Ant, 3, Color::Black) => 22,

            (Bug::Beetle, 1, Color::Black) => 23,
            (Bug::Beetle, 2, Color::Black) => 24,

            (Bug::Mosquito, 1, Color::Black) => 25,

            (Bug::Ladybug, 1, Color::Black) => 26,

            (Bug::Pillbug, 1, Color::Black) => 27,

            _ => {
                unreachable!()
            }
        }
    }

    fn get_connection_index(mut node_a: Node, mut node_b: Node, mut dir: EdgeDirection) -> usize {
        match dir {
            EdgeDirection::POINTY_NORTH_WEST
            | EdgeDirection::POINTY_SOUTH_WEST
            | EdgeDirection::POINTY_WEST => {
                (node_a, node_b) = (node_b, node_a);
                dir = dir >> 3;
            }
            _ => {}
        }

        let piece_num_a = Self::get_piece_number(node_a);
        let mut piece_num_b = Self::get_piece_number(node_b);
        if (piece_num_a < piece_num_b) {
            piece_num_b -= 1;
        }
        let dir_num = match dir {
            EdgeDirection::POINTY_NORTH_EAST => 0,
            EdgeDirection::POINTY_EAST => 1,
            EdgeDirection::POINTY_SOUTH_EAST => 2,
            _ => unreachable!(),
        };

        piece_num_a * 27 * 3 + piece_num_b * 3 + dir_num
    }

    fn get_position_index(mut node: Node, tile_id: usize) -> usize {
        let mut piece_num = Self::get_piece_number(node);
        piece_num * MAX_BOARD_SIZE + tile_id
    }

    fn get_height_index(node: Node) -> usize {
        let bug = node.get_bug().unwrap();
        let bug_num = node.get_bug_num();
        let color = node.get_color().unwrap();

        let offest_index = match (bug, bug_num, color) {
            (Bug::Beetle, 1, Color::White) => 0,
            (Bug::Beetle, 2, Color::White) => 1,
            (Bug::Mosquito, 1, Color::White) => 2,

            (Bug::Beetle, 1, Color::Black) => 3,
            (Bug::Beetle, 2, Color::Black) => 4,
            (Bug::Mosquito, 1, Color::Black) => 5,

            _ => unreachable!(),
        };
        6 * offest_index + EDGES_NNUE_BOARD
    }

    pub fn set_height(&mut self, node: Node, height: usize) {
        assert!(height > 0 && height < 8);
        let height_index = Self::get_height_index(node);

        for i in 0..6 {
            self.board[height_index + i] = false;
        }
        if height == 1 {
            return;
        }
        self.board[height_index + height - 2] = true;
    }

    pub fn set_connection(&mut self, node_a: Node, node_b: Node, dir: EdgeDirection) {
        let connection_index = Self::get_connection_index(node_a, node_b, dir);
        self.board[connection_index] = true;
    }

    pub fn set_position(&mut self, node: Node, tile_id: usize) {
        let position_index = Self::get_position_index(node, tile_id);
        self.board[position_index] = true;
    }

    pub fn unset_connection(&mut self, node_a: Node, node_b: Node, dir: EdgeDirection) {
        let connection_index = Self::get_connection_index(node_a, node_b, dir);
        self.board[connection_index] = false;
    }

    pub fn unset_position(&mut self, node: Node, tile_id: usize) {
        let position_index = Self::get_position_index(node, tile_id);
        self.board[position_index] = false;
    }

    pub fn get_connection(&mut self, node_a: Node, node_b: Node, dir: EdgeDirection) -> bool {
        let connection_index = Self::get_connection_index(node_a, node_b, dir);
        self.board[connection_index]
    }

    pub fn get_position(&mut self, node: Node, tile_id: usize) -> bool {
        let position_index = Self::get_position_index(node, tile_id);
        self.board[position_index]
    }

    pub fn get_height(&mut self, node: Node, height: usize) -> bool {
        let height_index = Self::get_height_index(node);
        self.board[height_index + height - 2]
    }
}
