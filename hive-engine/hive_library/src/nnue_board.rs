use crate::bug::Bug;
use crate::{Color, Node};
use hexx::EdgeDirection;
pub const NNUE_BOARD: usize = 28 * 27 * 3 + 6 * 6;

#[derive(Debug, Clone)]
pub struct NNUEBoard {
    board: [bool; NNUE_BOARD],
}

impl NNUEBoard {
    pub fn new() -> Self {
        Self {
            board: [false; NNUE_BOARD],
        }
    }
    fn get_piece_number(node: Node) -> usize {
        let bug = node.get_bug().unwrap();
        let bug_num = node.get_bug_num();
        let color = node.get_color().unwrap();

        match (bug, bug_num, color) {
            (Bug::Queen, 1, Color::White) => 0,
            (Bug::Queen, 1, Color::Black) => 1,

            (Bug::Grasshopper, 1, Color::White) => 2,
            (Bug::Grasshopper, 1, Color::Black) => 3,
            (Bug::Grasshopper, 2, Color::White) => 4,
            (Bug::Grasshopper, 2, Color::Black) => 5,
            (Bug::Grasshopper, 3, Color::White) => 6,
            (Bug::Grasshopper, 3, Color::Black) => 7,

            (Bug::Spider, 1, Color::White) => 8,
            (Bug::Spider, 1, Color::Black) => 9,
            (Bug::Spider, 2, Color::White) => 10,
            (Bug::Spider, 2, Color::Black) => 11,

            (Bug::Ant, 1, Color::White) => 12,
            (Bug::Ant, 1, Color::Black) => 13,
            (Bug::Ant, 2, Color::White) => 14,
            (Bug::Ant, 2, Color::Black) => 15,
            (Bug::Ant, 3, Color::White) => 16,
            (Bug::Ant, 3, Color::Black) => 17,

            (Bug::Beetle, 1, Color::White) => 18,
            (Bug::Beetle, 1, Color::Black) => 19,
            (Bug::Beetle, 2, Color::White) => 20,
            (Bug::Beetle, 2, Color::Black) => 21,

            (Bug::Mosquito, 1, Color::White) => 22,
            (Bug::Mosquito, 1, Color::Black) => 23,

            (Bug::Ladybug, 1, Color::White) => 24,
            (Bug::Ladybug, 1, Color::Black) => 25,

            (Bug::Pillbug, 1, Color::White) => 26,
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

    fn get_height_index(node: Node) -> usize {
        let starting_index = 28 * 27 * 3;

        let bug = node.get_bug().unwrap();
        let bug_num = node.get_bug_num();
        let color = node.get_color().unwrap();

        let offest_index = match (bug, bug_num, color) {
            (Bug::Beetle, 1, Color::White) => 0,
            (Bug::Beetle, 1, Color::Black) => 1,
            (Bug::Beetle, 2, Color::White) => 2,
            (Bug::Beetle, 2, Color::Black) => 3,

            (Bug::Mosquito, 1, Color::White) => 4,
            (Bug::Mosquito, 1, Color::Black) => 5,

            _ => unreachable!(),
        };
        6 * offest_index + starting_index
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

    pub fn unset_connection(&mut self, node_a: Node, node_b: Node, dir: EdgeDirection) {
        let connection_index = Self::get_connection_index(node_a, node_b, dir);
        self.board[connection_index] = false;
    }

    pub fn get_connection(&mut self, node_a: Node, node_b: Node, dir: EdgeDirection) -> bool {
        let connection_index = Self::get_connection_index(node_a, node_b, dir);
        self.board[connection_index]
    }

    pub fn get_height(&mut self, node: Node, height: usize) -> bool {
        let height_index = Self::get_height_index(node);
        self.board[height_index + height - 2]
    }
}
