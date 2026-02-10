use hexx::EdgeDirection;
use crate::{Color, Node};
use crate::bug::Bug;

struct NNUEBoard;

impl NNUEBoard {
    pub fn get_piece_number(node:Node) -> usize {
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
            
            _ => unreachable!(),
        }
    }
    
    pub fn get_connection_index(mut node_a:Node, mut node_b:Node, mut dir: EdgeDirection) -> usize {
        match dir {
            EdgeDirection::POINTY_NORTH_WEST | EdgeDirection::POINTY_SOUTH_WEST | EdgeDirection::POINTY_WEST => {
                (node_a, node_b) = (node_b, node_a);
                dir = dir >> 3;
            }
            _ => {}
        }

        let piece_num_a = Self::get_piece_number(node_a);
        let mut piece_num_b = Self::get_piece_number(node_b);
        if(piece_num_a < piece_num_b) {
            piece_num_b -= 1;
        }
        let dir_num = match dir {
            EdgeDirection::POINTY_NORTH_EAST => 0,
            EdgeDirection::POINTY_EAST => 1,
            EdgeDirection::POINTY_SOUTH_EAST => 2,
            _ => unreachable!()
        };
        
        piece_num_a * 27 * 3 + piece_num_b * 3 + dir_num
    }
}