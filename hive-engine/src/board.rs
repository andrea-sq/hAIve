use std::cmp::min;
use std::hash::{DefaultHasher, Hasher};
use std::sync::OnceLock;
use bitfield::bitfield;
use color_eyre::owo_colors::OwoColorize;
use hexx::Hex;
use hexx::storage::HexModMap;
use crate::bug::Bug;

pub(crate) const START_HEX: Hex = Hex::ZERO;
pub(crate) const GRID_RADIUS: usize = 18;
pub(crate) const GRID_SIZE: usize = 919;

static ZORBIST_TABLE: OnceLock<[u64; GRID_SIZE * 2]> = OnceLock::new();

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Color {
    White = 0,
    Black = 1,
}

impl TryFrom<u8> for Color {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Color::White),
            1 => Ok(Color::Black),
            _ => Err(()),
        }
    }
}

impl Into<u8> for Color {
    fn into(self) -> u8 {
        self as u8
    }
}

impl Color {
    pub fn other(self) -> Self {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct Node(u8);
    impl Debug;
    u8, from try_into Color, get_color, set_color: 7, 7;
    u8, from try_into Bug, get_bug, set_bug: 6, 4;
    get_bug_num, set_bug_num: 3, 2;
    get_tile_height, set_tile_height: 1, 0;
}

impl Node {

    fn empty() -> Self {
        Node(0)
    }

    fn new_occupied(bug: Bug, color: Color, bug_num: u8, clipped_height: u8) -> Self {
        let mut node = Node(0);
        node.set_bug(bug);
        node.set_color(color);
        node.set_bug_num(bug_num);
        node.set_tile_height(clipped_height);
        node
    }

    fn occupied(&self) -> bool {
        self.0 != 0
    }
}

#[derive(Copy, Clone, Debug)]
pub struct UnderNode {
    node: Node,
    hex: Hex,
    height: u8
}

impl UnderNode {
    fn new(node: Node, hex: Hex, height: u8) -> Self {
        Self { node, hex, height }
    }

    fn empty() -> Self {
        Self { node: Node::empty(), hex: Hex::ZERO, height: 0 }
    }
    pub fn get_node(&self) -> Node {
        self.node
    }

    pub fn get_hex(&self) -> Hex {
        self.hex
    }
}

#[derive(Clone, Debug)]
pub struct Board {
    pub(crate) nodes: HexModMap<Node>,
    underworld: [UnderNode; 8],
    underworld_size: usize,
    pub(crate) remaining: [[u8; 8]; 2],
    pub(crate) queens: [Hex; 2],
    pub(crate) occupied_hexes: [Vec<Hex>; 2],

    pub(crate) turn_num: u16,
    zorbist_table: &'static [u64; GRID_SIZE * 2],
    zorbist_hash: u64,
    zorbist_history: Vec<u64>,

    pub(super) turn_history: Vec<Turn>,
    pub(super) game_type_bits: u8
}

impl Board {
    pub fn to_move(&self) -> Color {
        if self.turn_num.is_multiple_of(2) {
            Color::White
        } else {
            Color::Black
        }
    }

    fn zorbist(&self, hex: Hex, bug: Bug, color: Color, height: u8) -> u64 {
        let id_hex = hex.const_sub(self.nodes.bounds().center)
            .to_hexmod_coordinates(self.nodes.bounds().radius) as usize;
        let hash = self.zorbist_table[((id_hex << 1) | color as usize)];
        hash.rotate_left(((height as u32) << 3) | bug as u32)
    }

    pub(crate) fn node(&self, hex: Hex) -> Node {
        self.nodes[hex]
    }

    pub fn get_underworld(&self) -> &[UnderNode] {
        &self.underworld[..self.underworld_size]
    }

    fn underworld_height(&self, hex: Hex, node: Node) -> u8 {
        let height = node.get_tile_height();
        if height > 2 {
            1 + self.underworld[..self.underworld_size]
                .iter()
                .rev()
                .find(|under| under.hex == hex)
                .and_then(|under| Some(under.height))
                .unwrap_or(0)
        } else {
            height
        }
    }

    fn insert_underworld(&mut self, node: Node, hex: Hex) {
        let height = self.underworld_height(hex, node);
        if self.underworld_size >= self.underworld.len() {
            unreachable!("underworld is full");
        }
        self.underworld[self.underworld_size] = UnderNode::new(node, hex, height);
        self.underworld_size += 1;
    }

    fn remove_underworld(&mut self, hex: Hex) -> Node{
        for i in (0..self.underworld_size).rev() {
            if self.underworld[i].hex == hex {
                let node = self.underworld[i].node;
                self.underworld[i..self.underworld_size].rotate_left(1);
                self.underworld_size -= 1;
                return node;
            }
        }
        unreachable!("underworld does not contain hex");
    }

    fn occupied_add(&mut self, color: Color, hex: Hex) {
        self.occupied_hexes[color as usize].push(hex);
    }

    fn occupied_remove(&mut self, color: Color, hex: Hex) {
        let vec = &mut self.occupied_hexes[color as usize];
        let i = vec.iter().position(|&h| h == hex).unwrap();
        vec.swap_remove(i);
    }

    fn insert(&mut self, hex: Hex, bug: Bug, bug_num: u8, color: Color) {
        let prev = self.node(hex);
        if prev.occupied() {
            if prev.get_color().unwrap() != color {
                self.occupied_remove(prev.get_color().unwrap(), hex);
                self.occupied_add(color, hex);
            }
            self.insert_underworld(prev, hex);
        } else {
            self.occupied_add(color, hex);
        }
        let tile_height = min(3, prev.get_tile_height() + 1);
        self.nodes[hex] = Node::new_occupied(bug, color, bug_num, tile_height);
        self.zorbist_hash ^= self.zorbist(hex, bug, color, self.height(hex));

        if bug == Bug::Queen {
            self.queens[color as usize] = hex;
        }
    }

    fn height(&self, hex: Hex) -> u8 {
        self.underworld_height(hex, self.node(hex))
    }

    pub(crate) fn occupied(&self, hex: Hex) -> bool {
        self.node(hex).occupied()
    }

    pub(crate) fn get_remaining(&self) -> &[u8; 8] {
        &self.remaining[self.to_move() as usize]
    }

    pub(crate) fn get_opponent_remaining(&self) -> &[u8; 8] {
        &self.remaining[(self.to_move() as usize + 1) % 2]
    }

    fn mut_remaining(&mut self) -> &mut [u8; 8] {
        &mut self.remaining[self.to_move() as usize]
    }

    fn remove(&mut self, hex: Hex) -> (Bug, u8, Color) {
        let height = self.height(hex);
        let prev = self.node(hex);
        let new_node = if height > 1 {self.remove_underworld(hex)} else {Node::empty()};
        self.nodes[hex] = new_node;
        let bug = prev.get_bug().unwrap();
        let color = prev.get_color().unwrap();
        if new_node.occupied() {
            let new_color = new_node.get_color().unwrap();
            if new_color != color {
                self.occupied_remove(color, hex);
                self.occupied_add(new_color, hex);
            }
        } else {
            self.occupied_remove(color, hex);
        }
        self.zorbist_hash ^= self.zorbist(hex, bug, color, height);
        if bug == Bug::Queen {
            self.queens[color as usize] = START_HEX;
        }
        (bug, prev.get_bug_num(), color)
    }

    pub(crate) fn get_available_bugs(&self) -> [(Bug, u8); 8] {
        let remaining = self.get_remaining();
        [
            (Bug::Queen, remaining[Bug::Queen as usize]),
            (Bug::Grasshopper, remaining[Bug::Grasshopper as usize]),
            (Bug::Spider, remaining[Bug::Spider as usize]),
            (Bug::Ant, remaining[Bug::Ant as usize]),
            (Bug::Beetle, remaining[Bug::Beetle as usize]),
            (Bug::Mosquito, remaining[Bug::Mosquito as usize]),
            (Bug::Ladybug, remaining[Bug::Ladybug as usize]),
            (Bug::Pillbug, remaining[Bug::Pillbug as usize]),
        ]
    }

    pub(crate) fn is_queen_required(&self) -> bool {
        self.turn_num > 5 && self.get_remaining()[Bug::Queen as usize] > 0
    }

    pub(crate) fn queens_surrounded(&self) -> [usize; 2] {
        let mut out = [0; 2];
        for (i, entry) in out.iter_mut().enumerate() {
            *entry = self.queens[i].all_neighbors().iter().filter(|adj| self.occupied(**adj)).count();
        }
        out
    }

    pub(super) fn new(remaining: [u8; 8]) -> Self {
        let mut game_type_bits = 0u8;
        for (i, &remain) in remaining.iter().enumerate() {
            if remain > 0 {
                game_type_bits |= 1 << i;
            }
        }

        let zorbist_table = ZORBIST_TABLE.get_or_init(|| {
            let mut table = [0u64; GRID_SIZE * 2];
            let mut hasher = DefaultHasher::new();
            for (i, entry) in table.iter_mut().enumerate() {
                hasher.write_usize(i);
                *entry = hasher.finish();
            }
            table
        });
        Self {
            nodes: HexModMap::new(START_HEX, GRID_RADIUS as u32, |coord| Node(0)),
            underworld: [UnderNode::empty(); 8],
            underworld_size: 0,
            remaining: [remaining; 2],
            queens: [START_HEX; 2],
            occupied_hexes: [Vec::new(), Vec::new()],
            turn_num: 0,
            zorbist_table,
            zorbist_hash: 0,
            zorbist_history: Vec::new(),
            turn_history: Vec::new(),
            game_type_bits
        }
    }
    pub fn new_core_set() -> Self {
        Self::new([1, 3, 2, 3, 2, 0, 0, 0])
    }

    pub fn new_expansions() -> Self {
        Self::new([1, 3, 2, 3, 2, 1, 1, 1])
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new_expansions()
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum Turn {
    Place(Hex, Bug),
    Move(Hex, Hex),
    #[default]
    Pass
}

impl Board {
    pub fn apply(&mut self, turn: Turn) {
        match turn {
            Turn::Place(hex, bug) => {
                let bug_num = Bug::initial_quantity()[bug as usize] - self.get_remaining()[bug as usize] + 1;
                self.insert(hex, bug, bug_num, self.to_move());
                self.mut_remaining()[bug as usize] -= 1;
            },
            Turn::Move(from, to) => {
                let (bug, bug_num, color) = self.remove(from);
                self.insert(to, bug, bug_num, color);
            }
            _ => {}
        }
        self.turn_num += 1;
        self.zorbist_hash ^= 0xa6c11b626b105b7c;
        self.zorbist_history.push(self.zorbist_hash);
        self.turn_history.push(turn);
    }

    pub fn undo(&mut self, turn: Turn) {
        self.turn_num -=1;
        self.zorbist_history.pop();
        self.turn_history.pop();
        self.zorbist_hash ^= 0xa6c11b626b105b7c;
        match turn {
            Turn::Place(hex, bug) => {
                self.remove(hex);
                self.mut_remaining()[bug as usize] += 1;
            },
            Turn::Move(from, to) => {
                let (bug, bug_num, color) = self.remove(to);
                self.insert(from, bug, bug_num, color);
            }
            _ => {}
        }
    }


}


