use crate::Node;
use crate::board::GRID_SIZE;
use crate::hex_grid::neighbor_table::NEIGHBOR_TABLE;
use hexx::EdgeDirection;
use std::ops::IndexMut;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Hex(pub u16);

impl Hex {
    pub const ZERO: Hex = Hex(0);

    #[inline]
    pub fn neighbors(&self) -> &'static [Hex; 6] {
        &NEIGHBOR_TABLE[self.0 as usize]
    }
    pub fn neighbor(&self, dir: EdgeDirection) -> Hex {
        match dir {
            EdgeDirection::FLAT_SOUTH_EAST => NEIGHBOR_TABLE[self.0 as usize][0],
            EdgeDirection::FLAT_SOUTH => NEIGHBOR_TABLE[self.0 as usize][1],
            EdgeDirection::FLAT_SOUTH_WEST => NEIGHBOR_TABLE[self.0 as usize][2],
            EdgeDirection::FLAT_NORTH_WEST => NEIGHBOR_TABLE[self.0 as usize][3],
            EdgeDirection::FLAT_NORTH => NEIGHBOR_TABLE[self.0 as usize][4],
            EdgeDirection::FLAT_NORTH_EAST => NEIGHBOR_TABLE[self.0 as usize][5],
            _ => unreachable!(),
        }
    }
}

mod neighbor_table;

#[derive(Clone, Debug)]
pub struct HexGrid {
    id_table: &'static [[usize; 29]; 29],
    pub hex_table: &'static [hexx::Hex; GRID_SIZE],
    out_of_map_table: &'static [[hexx::Hex; 31]; 31],
    grid: [Node; GRID_SIZE],
}

impl HexGrid {
    pub fn new(
        id_table: &'static [[usize; 29]; 29],
        out_of_map_table: &'static [[hexx::Hex; 31]; 31],
        hex_table: &'static [hexx::Hex; GRID_SIZE],
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
            .map(|(i, node)| (Hex(i as u16), node))
    }
}

impl std::ops::Index<Hex> for HexGrid {
    type Output = Node;

    fn index(&self, index: Hex) -> &Self::Output {
        &self.grid[index.0 as usize]
    }
}

impl IndexMut<Hex> for HexGrid {
    fn index_mut(&mut self, index: Hex) -> &mut Self::Output {
        &mut self.grid[index.0 as usize]
    }
}
