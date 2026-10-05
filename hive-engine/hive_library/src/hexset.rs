use crate::board::GRID_SIZE;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HexSet {
    hexes: [bool; GRID_SIZE],
}

impl HexSet {
    pub fn new() -> Self {
        Self {
            hexes: [false; GRID_SIZE],
        }
    }
    pub fn insert(&mut self, hex: usize) {
        self.hexes[hex] = true;
    }
    pub fn remove(&mut self, hex: usize) {
        self.hexes[hex] = false;
    }
    pub fn contains(&self, hex: usize) -> bool {
        self.hexes[hex]
    }
    pub fn clear(&mut self) {
        self.hexes.iter_mut().for_each(|x| *x = false);
    }
}
