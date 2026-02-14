use crate::bug::Bug;
use crate::{Color, Node};

pub const MAX_ACTIVE_FEATURES: usize = 16 + 6;
pub const MAX_BOARD_SIZE: usize = 3 * 14 + 3 * 14 * 14 + 1;
const EDGES_NNUE_BOARD: usize = MAX_BOARD_SIZE * 16;
const HEIGHTS_NNUE_BOARD: usize = 6 * 6;
pub const NNUE_BOARD: usize = EDGES_NNUE_BOARD + HEIGHTS_NNUE_BOARD;

#[derive(Debug, Clone)]
pub struct NNUEBoard {
    board: [bool; NNUE_BOARD],
    board_black: [bool; NNUE_BOARD],
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
            board_black: [false; NNUE_BOARD],
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
        self.board_black
            .iter()
            .enumerate()
            .filter_map(|(i, b)| if *b { Some(i) } else { None })
            .collect()
        // let half_edges = EDGES_NNUE_BOARD / 2;
        // let half_heights = HEIGHTS_NNUE_BOARD / 2;
        //
        // // TODO: invert internals
        // self.board[half_edges..EDGES_NNUE_BOARD]
        //     .iter()
        //     .chain(self.board[0..half_edges].iter())
        //     .chain(self.board[(EDGES_NNUE_BOARD + half_heights)..NNUE_BOARD].iter())
        //     .chain(self.board[EDGES_NNUE_BOARD..(EDGES_NNUE_BOARD + half_heights)].iter())
        //     .enumerate()
        //     .filter_map(|(i, b)| if *b { Some(i) } else { None })
        //     .collect()
    }

    fn get_piece_number(node: Node) -> usize {
        let bug = node.get_bug().unwrap();
        let bug_num = node.get_bug_num();
        let color = node.get_color().unwrap();

        match (bug, color) {
            //WHITE
            (Bug::Queen, Color::White) => 0,

            (Bug::Grasshopper, Color::White) => 1,

            (Bug::Spider, Color::White) => 2,

            (Bug::Ant, Color::White) => 3,
            (Bug::Beetle, Color::White) => 4,

            (Bug::Mosquito, Color::White) => 5,

            (Bug::Ladybug, Color::White) => 6,

            (Bug::Pillbug, Color::White) => 7,
            //BLACK
            (Bug::Queen, Color::Black) => 8,

            (Bug::Grasshopper, Color::Black) => 9,

            (Bug::Spider, Color::Black) => 10,

            (Bug::Ant, Color::Black) => 11,
            (Bug::Beetle, Color::Black) => 12,

            (Bug::Mosquito, Color::Black) => 13,

            (Bug::Ladybug, Color::Black) => 14,

            (Bug::Pillbug, Color::Black) => 15,

            _ => {
                unreachable!()
            }
        }
    }

    // fn get_connection_index(mut node_a: Node, mut node_b: Node, mut dir: EdgeDirection) -> usize {
    //     match dir {
    //         EdgeDirection::POINTY_NORTH_WEST
    //         | EdgeDirection::POINTY_SOUTH_WEST
    //         | EdgeDirection::POINTY_WEST => {
    //             (node_a, node_b) = (node_b, node_a);
    //             dir = dir >> 3;
    //         }
    //         _ => {}
    //     }
    //
    //     let piece_num_a = Self::get_piece_number(node_a);
    //     let mut piece_num_b = Self::get_piece_number(node_b);
    //     if (piece_num_a < piece_num_b) {
    //         piece_num_b -= 1;
    //     }
    //     let dir_num = match dir {
    //         EdgeDirection::POINTY_NORTH_EAST => 0,
    //         EdgeDirection::POINTY_EAST => 1,
    //         EdgeDirection::POINTY_SOUTH_EAST => 2,
    //         _ => unreachable!(),
    //     };
    //
    //     piece_num_a * 27 * 3 + piece_num_b * 3 + dir_num
    // }

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
        let mut node_black = node.clone();
        node_black.set_color((1 - node.get_color().unwrap() as u8).try_into().unwrap());
        let height_black_index = Self::get_height_index(node_black);

        for i in 0..6 {
            self.board[height_index + i] = false;
            self.board_black[height_black_index + i] = false;
        }
        if height == 1 {
            return;
        }
        self.board[height_index + height - 2] = true;
        self.board[height_black_index + height - 2] = true;
    }

    // pub fn set_connection(&mut self, node_a: Node, node_b: Node, dir: EdgeDirection) {
    //     let connection_index = Self::get_connection_index(node_a, node_b, dir);
    //     self.board[connection_index] = true;
    // }

    pub fn set_position(&mut self, node: Node, tile_id: usize) {
        let position_index = Self::get_position_index(node, tile_id);
        let mut node_black = node.clone();
        node_black.set_color((1 - node.get_color().unwrap() as u8).try_into().unwrap());
        let position_black_index = Self::get_position_index(node_black, tile_id);
        self.board[position_index] = true;
        self.board_black[position_black_index] = true;
    }

    // pub fn unset_connection(&mut self, node_a: Node, node_b: Node, dir: EdgeDirection) {
    //     let connection_index = Self::get_connection_index(node_a, node_b, dir);
    //     self.board[connection_index] = false;
    // }

    pub fn unset_position(&mut self, node: Node, tile_id: usize) {
        let position_index = Self::get_position_index(node, tile_id);
        let mut node_black = node.clone();
        node_black.set_color((1 - node.get_color().unwrap() as u8).try_into().unwrap());
        let position_black_index = Self::get_position_index(node_black, tile_id);
        self.board[position_index] = false;
        self.board_black[position_black_index] = false;
    }

    // pub fn get_connection(&mut self, node_a: Node, node_b: Node, dir: EdgeDirection) -> bool {
    //     let connection_index = Self::get_connection_index(node_a, node_b, dir);
    //     self.board[connection_index]
    // }

    pub fn get_position(&mut self, node: Node, tile_id: usize) -> bool {
        let position_index = Self::get_position_index(node, tile_id);
        self.board[position_index]
    }

    pub fn get_height(&mut self, node: Node, height: usize) -> bool {
        let height_index = Self::get_height_index(node);
        self.board[height_index + height - 2]
    }
}
