use crate::board::GRID_SIZE;
use crate::Node;
use hexx::Hex;
use std::ops::IndexMut;

#[derive(Clone, Debug)]
pub struct HexGrid {
    id_table: &'static [[usize; 29]; 29],
    hex_table: &'static [Hex; GRID_SIZE],
    out_of_map_table: &'static [[Hex; 31]; 31],
    grid: [Node; GRID_SIZE],
}

impl HexGrid {
    pub fn new(
        id_table: &'static [[usize; 29]; 29],
        out_of_map_table: &'static [[Hex; 31]; 31],
        hex_table: &'static [Hex; GRID_SIZE],
    ) -> Self {
        let grid = [Node::empty(); GRID_SIZE];
        HexGrid {
            grid,
            id_table,
            hex_table,
            out_of_map_table,
        }
    }
    pub fn iter(&self) -> impl Iterator<Item = (Hex, &Node)> {
        self.grid
            .iter()
            .enumerate()
            .map(|(i, node)| (self.out_of_map_table[i][0], node))
    }
}

impl std::ops::Index<hexx::Hex> for HexGrid {
    type Output = Node;

    fn index(&self, index: Hex) -> &Self::Output {
        &self.grid[self.id_table[(index.x + 14) as usize][(index.y + 14) as usize]]
    }
}

impl IndexMut<Hex> for HexGrid {
    fn index_mut(&mut self, index: Hex) -> &mut Self::Output {
        &mut self.grid[self.id_table[(index.x + 14) as usize][(index.y + 14) as usize]]
    }
}
