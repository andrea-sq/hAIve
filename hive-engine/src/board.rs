use crate::bug::Bug;
use crate::hexset::HexSet;
use bitfield::bitfield;
use hexx::storage::HexModMap;
use hexx::{EdgeDirection, Hex, HexBounds};
use minimax::Winner;
use std::cmp::{max, min};
use std::collections::HashSet;
use std::hash::{DefaultHasher, Hasher};
use std::sync::OnceLock;

pub(crate) const START_HEX: Hex = Hex::ZERO;
pub(crate) const GRID_RADIUS: usize = 18;
pub(crate) const GRID_SIZE: usize = 1027;

static ZOBRIST_TABLE: OnceLock<[u64; GRID_SIZE * 2]> = OnceLock::new();

pub fn neighbor(hex: Hex, dir: EdgeDirection, hex_bounds: HexBounds) -> Hex {
    hex.neighbor(dir).const_sub(hex_bounds.center)
}

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

impl From<Color> for u8 {
    fn from(value: Color) -> Self {
        value as u8
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
    pub u8, from try_into Color, get_color, set_color: 7, 7;
    pub u8, from try_into Bug, get_bug, set_bug: 6, 4;
    pub get_bug_num, set_bug_num: 3, 2;
    pub get_tile_height, set_tile_height: 1, 0;
}

impl Node {
    pub(crate) fn empty() -> Self {
        Node(0)
    }

    pub(crate) fn new_occupied(bug: Bug, color: Color, bug_num: u8, clipped_height: u8) -> Self {
        let mut node = Node(0);
        node.set_bug(bug);
        node.set_color(color);
        node.set_bug_num(bug_num);
        node.set_tile_height(clipped_height);
        node
    }

    pub(crate) fn occupied(&self) -> bool {
        self.0 != 0
    }

    pub(crate) fn is_stacked(self) -> bool {
        self.get_tile_height() > 1
    }
}

#[derive(Copy, Clone, Debug)]
pub struct UnderNode {
    node: Node,
    hex: Hex,
    height: u8,
}

impl UnderNode {
    fn new(node: Node, hex: Hex, height: u8) -> Self {
        Self { node, hex, height }
    }

    fn empty() -> Self {
        Self {
            node: Node::empty(),
            hex: Hex::ZERO,
            height: 0,
        }
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
    pub(super) game_type_bits: u8,
}

impl Board {
    pub fn find_id(&self, hex: Hex) -> usize {
        hex.const_sub(self.nodes.bounds().center)
            .to_hexmod_coordinates(self.nodes.bounds().radius) as usize
    }

    pub fn adjacent(&self, hex: Hex) -> [Hex; 6] {
        let hex_bounds = self.nodes.bounds();
        let mut res = hex.all_neighbors();
        res.iter_mut()
            .for_each(|h| *h = h.const_sub(hex_bounds.center));
        res
    }
    pub fn neighbor(&self, hex: Hex, dir: EdgeDirection) -> Hex {
        let hex_bounds = self.nodes.bounds();
        hex.neighbor(dir).const_sub(hex_bounds.center)
    }

    pub fn to_move(&self) -> Color {
        if self.turn_num.is_multiple_of(2) {
            Color::White
        } else {
            Color::Black
        }
    }

    fn zorbist(&self, hex: Hex, bug: Bug, color: Color, height: u8) -> u64 {
        let id_hex = hex
            .const_sub(self.nodes.bounds().center)
            .to_hexmod_coordinates(self.nodes.bounds().radius) as usize;
        let hash = self.zorbist_table[(id_hex << 1) | color as usize];
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
                .map(|under| under.height)
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

    fn remove_underworld(&mut self, hex: Hex) -> Node {
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
        let new_node = if height > 1 {
            self.remove_underworld(hex)
        } else {
            Node::empty()
        };
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
            *entry = self
                .adjacent(self.queens[i])
                .iter()
                .filter(|adj| self.occupied(**adj))
                .count();
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

        let zorbist_table = ZOBRIST_TABLE.get_or_init(|| {
            let mut table = [0u64; GRID_SIZE * 2];
            let mut hasher = DefaultHasher::new();
            for (i, entry) in table.iter_mut().enumerate() {
                hasher.write_usize(i);
                *entry = hasher.finish();
            }
            table
        });
        Self {
            nodes: HexModMap::new(START_HEX, GRID_RADIUS as u32, |_| Node(0)),
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
            game_type_bits,
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
    Pass,
}

impl Board {
    pub fn apply(&mut self, turn: Turn) {
        match turn {
            Turn::Place(hex, bug) => {
                let bug_num =
                    Bug::initial_quantity()[bug as usize] - self.get_remaining()[bug as usize] + 1;
                self.insert(hex, bug, bug_num, self.to_move());
                self.mut_remaining()[bug as usize] -= 1;
            }
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
        self.turn_num -= 1;
        self.zorbist_history.pop();
        self.turn_history.pop();
        self.zorbist_hash ^= 0xa6c11b626b105b7c;
        match turn {
            Turn::Place(hex, bug) => {
                self.remove(hex);
                self.mut_remaining()[bug as usize] += 1;
            }
            Turn::Move(from, to) => {
                let (bug, bug_num, color) = self.remove(to);
                self.insert(from, bug, bug_num, color);
            }
            _ => {}
        }
    }
}

impl Board {
    fn generate_placements(&self, turns: &mut Vec<Turn>) {
        let mut no_placement = HexSet::new();
        for &enemy in self.occupied_hexes[self.to_move().other() as usize].iter() {
            for adj in self.adjacent(enemy) {
                no_placement.insert(self.find_id(adj));
            }
        }
        for &friend in self.occupied_hexes[self.to_move() as usize].iter() {
            for hex in self.adjacent(friend) {
                if no_placement.contains(self.find_id(hex)) {
                    continue;
                }
                no_placement.insert(self.find_id(hex));
                if self.occupied(hex) {
                    continue;
                }
                for (bug, num_left) in self.get_available_bugs().iter() {
                    if self.is_queen_required() && *bug != Bug::Queen {
                        continue;
                    }
                    if *num_left > 0 {
                        turns.push(Turn::Place(hex, *bug));
                    }
                }
            }
        }
    }

    pub(crate) fn find_cut_vertices(&self) -> HexSet {
        struct State<'a> {
            board: &'a Board,
            visited: HexSet,
            immovable: HexSet,
            num: [u8; GRID_SIZE],
            low: [u8; GRID_SIZE],
            visit_num: u8,
        }

        let mut state = State {
            board: self,
            visited: HexSet::new(),
            immovable: HexSet::new(),
            num: [0; GRID_SIZE],
            low: [0; GRID_SIZE],
            visit_num: 1,
        };

        fn dfs(state: &mut State, hex: Hex, parent: Hex) {
            state.visited.insert(state.board.find_id(hex));
            let id_hex = hex
                .const_sub(state.board.nodes.bounds().center)
                .to_hexmod_coordinates(state.board.nodes.bounds().radius)
                as usize;
            state.num[id_hex] = state.visit_num;
            state.low[id_hex] = state.visit_num;
            state.visit_num += 1;

            let root = hex == parent;
            let mut children = 0;
            for adj in state.board.adjacent(hex) {
                let id_adj = adj
                    .const_sub(state.board.nodes.bounds().center)
                    .to_hexmod_coordinates(state.board.nodes.bounds().radius)
                    as usize;
                if !state.board.occupied(adj) {
                    continue;
                }
                if adj == parent {
                    continue;
                }
                if state.visited.contains(state.board.find_id(adj)) {
                    state.low[id_hex] = min(state.low[id_hex], state.num[id_adj]);
                } else {
                    dfs(state, adj, hex);
                    state.low[id_hex] = min(state.low[id_hex], state.low[id_adj]);
                    if state.low[id_adj] >= state.num[id_hex] && !root {
                        state.immovable.insert(state.board.find_id(hex));
                    }
                    children += 1;
                }
            }
            if root && children > 1 {
                state.immovable.insert(state.board.find_id(hex));
            }
        }
        let start = self.queens[0];
        dfs(&mut state, start, start);
        state.immovable
    }

    pub(crate) fn slideable_adjacent<'a>(
        &self,
        neighbors: &'a mut [Hex; 6],
        origin: Hex,
        hex: Hex,
    ) -> impl Iterator<Item = Hex> + 'a {
        *neighbors = self.adjacent(hex);
        let mut occupied = 0;
        for neighbor in neighbors.iter().rev() {
            occupied <<= 1;
            if self.occupied(*neighbor) && *neighbor != origin {
                occupied |= 1;
            }
        }
        occupied |= (occupied << 6) | (occupied << 12);
        let slideable = (!occupied & ((occupied << 1) ^ (occupied >> 1))) >> 6;

        neighbors.iter().enumerate().filter_map(move |(i, &hex)| {
            if (slideable >> i) & 1 != 0 {
                Some(hex)
            } else {
                None
            }
        })
    }

    fn slideable_adjacent_beetle<'a>(
        &self,
        out: &'a mut [Hex; 6],
        orig: Hex,
        hex: Hex,
    ) -> impl Iterator<Item = Hex> + 'a {
        let mut self_height = self.height(hex);
        if orig == hex {
            self_height -= 1;
        }
        let mut heights = [0; 6];
        let neighbors = self.adjacent(hex);
        for i in 0..6 {
            heights[i] = self.height(neighbors[i]);
        }

        let mut n = 0;
        for i in 0..6 {
            let barrier = max(self_height, heights[i]);
            if barrier == 0 {
                continue;
            }
            if heights[(i + 1) % 6] > barrier && heights[(i + 5) % 6] > barrier {
                continue;
            }
            out[n] = neighbors[i];
            n += 1;
        }

        out.iter().take(n).copied()
    }

    fn generate_stack_walking(&self, hex: Hex, turns: &mut Vec<Turn>) {
        let mut buf = [Hex::ZERO; 6];
        for adj in self.slideable_adjacent_beetle(&mut buf, hex, hex) {
            turns.push(Turn::Move(hex, adj));
        }
    }

    fn generate_jumps(&self, hex: Hex, turns: &mut Vec<Turn>) {
        for dir in EdgeDirection::ALL_DIRECTIONS {
            let mut jump = self.neighbor(hex, dir);
            let mut dist = 1;
            while self.occupied(jump) {
                jump = self.neighbor(jump, dir);
                dist += 1;
                if jump == hex {
                    dist = 0;
                    break;
                }
            }
            if dist > 1 {
                turns.push(Turn::Move(hex, jump));
            }
        }
    }

    fn generate_walk1(&self, hex: Hex, turns: &mut Vec<Turn>) {
        let mut buf = [Hex::ZERO; 6];
        for adj in self.slideable_adjacent(&mut buf, hex, hex) {
            turns.push(Turn::Move(hex, adj));
        }
    }

    fn generate_walk3(&self, orig: Hex, turns: &mut Vec<Turn>) {
        let mut buf1 = [Hex::ZERO; 6];
        let mut buf2 = [Hex::ZERO; 6];
        let mut buf3 = [Hex::ZERO; 6];
        let mut visited = HashSet::new();
        visited.insert(orig);

        for s1 in self.slideable_adjacent(&mut buf1, orig, orig) {
            for s2 in self.slideable_adjacent(&mut buf2, orig, s1) {
                if s2 != orig {
                    for s3 in self.slideable_adjacent(&mut buf3, orig, s2) {
                        if s3 != s1 && !visited.contains(&s3) {
                            turns.push(Turn::Move(orig, s3));
                            visited.insert(s3);
                        }
                    }
                }
            }
        }
    }

    fn generate_walk_all(&self, orig: Hex, turns: &mut Vec<Turn>) {
        let mut visited = HashSet::new();
        let mut queue = [Hex::ZERO; 16];
        queue[0] = orig;
        let mut qsize = 1;
        let mut buf = [Hex::ZERO; 6];
        while qsize > 0 {
            qsize -= 1;
            let node = queue[qsize];
            if visited.contains(&node) {
                continue;
            }
            visited.insert(node);
            if node != orig {
                turns.push(Turn::Move(orig, node));
            }
            for adj in self.slideable_adjacent(&mut buf, orig, node) {
                if !visited.contains(&adj) {
                    queue[qsize] = adj;
                    qsize += 1;
                }
            }
        }
    }
    fn generate_ladybug(&self, hex: Hex, turns: &mut Vec<Turn>) {
        let mut buf1 = [Hex::ZERO; 6];
        let mut buf2 = [Hex::ZERO; 6];
        let mut buf3 = [Hex::ZERO; 6];
        let mut step2 = HashSet::new();
        let mut step3 = HashSet::new();
        for s1 in self.slideable_adjacent_beetle(&mut buf1, hex, hex) {
            if self.occupied(s1) {
                for s2 in self.slideable_adjacent_beetle(&mut buf2, hex, s1) {
                    if self.occupied(s2) && !step2.contains(&s2) {
                        step2.insert(s2);
                        for s3 in self.slideable_adjacent_beetle(&mut buf3, hex, s2) {
                            if !self.occupied(s3) && !step3.contains(&s3) {
                                step3.insert(s3);
                                turns.push(Turn::Move(hex, s3));
                            }
                        }
                    }
                }
            }
        }
    }

    fn generate_throws(
        &self,
        immovable: &HexSet,
        hex: Hex,
        turns: &mut Vec<Turn>,
        throw_starts: &mut HexSet,
        throw_ends: &mut HexSet,
    ) {
        let mut starts = [Hex::ZERO; 6];
        let mut num_starts = 0;
        let mut ends = [Hex::ZERO; 6];
        let mut num_ends = 0;
        let mut buf = [Hex::ZERO; 6];
        let nw_direction = EdgeDirection::FLAT_NORTH_WEST;
        let origin = self.neighbor(self.neighbor(hex, nw_direction), nw_direction);
        for adj in self.slideable_adjacent_beetle(&mut buf, origin, hex) {
            match self.height(adj) {
                0 => {
                    ends[num_ends] = adj;
                    num_ends += 1;
                }
                1 => {
                    if !immovable.contains(self.find_id(adj)) {
                        starts[num_starts] = adj;
                        num_starts += 1;
                    }
                }
                _ => {}
            }
        }
        for &start in starts[..num_starts].iter() {
            for &end in ends[..num_ends].iter() {
                turns.push(Turn::Move(start, end));
                throw_starts.insert(self.find_id(start));
                throw_ends.insert(self.find_id(end));
            }
        }
    }

    fn generate_mosquito(&self, hex: Hex, turns: &mut Vec<Turn>) {
        let mut targets = [false; 8];
        for adj in self.adjacent(hex) {
            let node = self.nodes[adj];
            if node.occupied() {
                targets[node.get_bug().unwrap() as usize] = true;
            }
        }

        let mut i = turns.len();
        if targets[Bug::Ant as usize] {
            self.generate_walk_all(hex, turns);
        } else {
            // Avoid adding strictly duplicative moves to the ant.
            if targets[Bug::Queen as usize]
                || targets[Bug::Beetle as usize]
                || targets[Bug::Pillbug as usize]
            {
                self.generate_walk1(hex, turns);
            }
            if targets[Bug::Spider as usize] {
                self.generate_walk3(hex, turns);
            }
        }
        if targets[Bug::Grasshopper as usize] {
            self.generate_jumps(hex, turns);
        }
        if targets[Bug::Beetle as usize] {
            self.generate_stack_walking(hex, turns);
        }
        if targets[Bug::Ladybug as usize] {
            self.generate_ladybug(hex, turns);
        }

        // Remove duplicates.
        let mut dests = HashSet::new();
        while i < turns.len() {
            if let Turn::Move(_, dest) = turns[i] {
                if dests.contains(&dest) {
                    turns.swap_remove(i);
                } else {
                    dests.insert(dest);
                    i += 1;
                }
            }
        }
    }

    pub(crate) fn generate_movements(&self, turns: &mut Vec<Turn>) {
        let mut immovable = self.find_cut_vertices();
        let stunned = match self.turn_history.last() {
            Some(Turn::Move(_, dest)) => Some(dest),
            _ => None,
        };
        if let Some(moved) = stunned {
            // Can't move pieces that were moved on the opponent's turn.
            immovable.insert(self.find_id(*moved));
        }

        // Pillbug throws need to be deduped against organic movements, so generate them first.
        let mut throw_starts = HexSet::new();
        let mut throw_ends = HexSet::new();
        let first_move = turns.len();
        let mut marker;
        for &hex in self.occupied_hexes[self.to_move() as usize].iter() {
            marker = turns.len();
            let node = self.node(hex);
            if stunned == Some(&hex) {
                continue;
            }
            if node.get_bug().unwrap() == Bug::Pillbug
                || (node.get_bug().unwrap() == Bug::Mosquito
                    && !node.is_stacked()
                    && self.adjacent(hex).iter().any(|&adj| {
                        let n = self.node(adj);
                        n.occupied() && n.get_bug().unwrap() == Bug::Pillbug
                    }))
            {
                self.generate_throws(&immovable, hex, turns, &mut throw_starts, &mut throw_ends);
                // Dedup throws from pillbug and mosquito
                if marker > 0 {
                    let mut i = marker;
                    while i < turns.len() {
                        if turns[first_move..marker].contains(&turns[i]) {
                            turns.swap_remove(i);
                        } else {
                            i += 1;
                        }
                    }
                }
            }
        }
        let num_throws = turns.len();

        for &hex in self.occupied_hexes[self.to_move() as usize].iter() {
            marker = turns.len();
            let node = self.node(hex);
            if node.is_stacked() {
                self.generate_stack_walking(hex, turns);
                continue;
            }
            if immovable.contains(self.find_id(hex)) {
                continue;
            }
            match node.get_bug().unwrap() {
                Bug::Queen => self.generate_walk1(hex, turns),
                Bug::Grasshopper => self.generate_jumps(hex, turns),
                Bug::Spider => self.generate_walk3(hex, turns),
                Bug::Ant => self.generate_walk_all(hex, turns),
                Bug::Beetle => {
                    self.generate_walk1(hex, turns);
                    self.generate_stack_walking(hex, turns);
                }
                Bug::Mosquito => self.generate_mosquito(hex, turns),
                Bug::Ladybug => self.generate_ladybug(hex, turns),
                Bug::Pillbug => self.generate_walk1(hex, turns),
            }

            // Dedup against pillbug throws.
            if throw_starts.contains(self.find_id(hex)) {
                let mut i = marker;
                while i < turns.len() {
                    let turn = turns[i];
                    let end = match turn {
                        Turn::Move(_, end) => end,
                        _ => {
                            i += 1;
                            continue;
                        }
                    };
                    if throw_ends.contains(self.find_id(end))
                        && turns[first_move..num_throws].contains(&turn)
                    {
                        turns.swap_remove(i);
                    } else {
                        i += 1;
                    }
                }
            }
        }
    }
}
pub struct Rules;

impl minimax::Game for Rules {
    type S = Board;
    type M = Turn;

    fn generate_moves(board: &Self::S, turns: &mut Vec<Self::M>) {
        if board.turn_num < 2 {
            for (bug, num_left) in board.get_available_bugs().iter() {
                if *bug == Bug::Queen {
                    continue;
                }
                if *num_left > 0 {
                    if board.turn_num == 0 {
                        turns.push(Turn::Place(START_HEX, *bug));
                    } else {
                        for &hex in board.adjacent(START_HEX).iter() {
                            turns.push(Turn::Place(hex, *bug));
                        }
                    }
                }
            }
            return;
        }

        if board.get_remaining()[Bug::Queen as usize] == 0 {
            board.generate_movements(turns);
        }

        if board.get_remaining().iter().any(|&n| n > 0) {
            board.generate_placements(turns);
        }

        if turns.is_empty() {
            turns.push(Turn::Pass);
        }
    }

    fn apply(board: &mut Self::S, turn: Self::M) -> Option<Self::S> {
        board.apply(turn);
        None
    }

    fn undo(board: &mut Self::S, turn: Self::M) {
        board.undo(turn);
    }
    fn get_winner(board: &Self::S) -> Option<Winner> {
        let queens_surrounded = board.queens_surrounded();
        let n = board.zorbist_history.len();
        if n > 10 {
            let position_repeat_count = board
                .zorbist_history
                .iter()
                .rev()
                .step_by(4)
                .skip(1)
                .take(8)
                .filter(|&&hash| hash == board.zorbist_hash)
                .count();
            if position_repeat_count >= 2 {
                return Some(Winner::Draw);
            }
        }

        if queens_surrounded == [6, 6] {
            Some(Winner::Draw)
        } else if queens_surrounded[board.to_move() as usize] == 6 {
            Some(Winner::PlayerJustMoved)
        } else if queens_surrounded[board.to_move().other() as usize] == 6 {
            Some(Winner::PlayerToMove)
        } else {
            None
        }
    }
    fn zobrist_hash(board: &Self::S) -> u64 {
        let mut hash = board.zorbist_hash;
        if let Some(Turn::Move(_, end)) = board.turn_history.last() {
            let id_end = board.find_id(*end);
            hash ^= id_end as u64;
        }

        hash
    }
    fn null_move(_board: &Self::S) -> Option<Self::M> {
        Some(Turn::Pass)
    }

    fn notation(board: &Self::S, turn: Self::M) -> Option<String> {
        Some(board.to_move_string(turn))
    }

    fn table_index(turn: Self::M) -> u16 {
        let hex_bounds = HexBounds::new(START_HEX, GRID_RADIUS as u32);

        // Arbitrary mask
        const MASK: u16 = 0x7f;
        match turn {
            Turn::Place(hex, bug) => {
                let hex_id = hex
                    .const_sub(START_HEX)
                    .to_hexmod_coordinates(GRID_RADIUS as u32);
                ((bug as u16) << 7) | hex_id as u16 & MASK
            }
            Turn::Move(start, end) => {
                let start_id = start
                    .const_sub(START_HEX)
                    .to_hexmod_coordinates(GRID_RADIUS as u32);
                let end_id = end
                    .const_sub(START_HEX)
                    .to_hexmod_coordinates(GRID_RADIUS as u32);
                ((start_id as u16 & MASK) << 7) | (end_id as u16 & MASK)
            }
            Turn::Pass => 0,
        }
    }

    fn max_table_index() -> u16 {
        u16::MAX >> 2
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hexx::hex;
    use hexx::storage::HexStore;
    use minimax::Game;

    impl Board {
        fn insert_loc(&mut self, loc: Hex, bug: Bug, color: Color) {
            self.insert(loc, bug, 0, color);
        }

        fn remove_loc(&mut self, loc: Hex) {
            self.remove(loc);
        }

        fn fill_board(&mut self, locs: &[Hex], bug: Bug) {
            for &loc in locs {
                self.insert(loc, bug, 0, Color::Black);
            }
        }

        fn assert_placements(&self, turns: &[Turn], expected: &[(Hex, Bug)]) {
            let mut actual_pairs = Vec::new();
            for &m in turns.iter() {
                if let Turn::Place(actual_hex, actual_bug) = m {
                    actual_pairs.push((actual_hex, actual_bug));
                }
            }
            let sorting_function = |&a: &(Hex, Bug), &b: &(Hex, Bug)| {
                self.find_id(a.0)
                    .cmp(&self.find_id(b.0))
                    .then(a.1.cmp(&b.1))
            };
            actual_pairs.sort_by(sorting_function);
            let mut expected_pairs = Vec::new();
            expected_pairs.extend(expected);
            expected_pairs.sort_by(sorting_function);
            assert_eq!(actual_pairs, expected_pairs);
        }

        fn assert_movements(&self, turns: &[Turn], start: Hex, ends: &[Hex]) {
            let mut actual_ends = Vec::new();
            for &m in turns.iter() {
                if let Turn::Move(actual_start, actual_end) = m
                    && actual_start == start
                {
                    actual_ends.push(actual_end);
                }
            }
            let sorting_function = |&a: &Hex, &b: &Hex| self.find_id(a).cmp(&self.find_id(b));
            actual_ends.sort_by(sorting_function);
            let mut expected_ends = Vec::new();
            expected_ends.extend(ends);
            expected_ends.sort_by(sorting_function);
            assert_eq!(actual_ends, expected_ends);
        }
    }

    #[test]
    fn test_gen_placement() {
        let mut board = Board::default();
        for i in 1..8 {
            board.remaining[0][i] = 0;
            board.remaining[1][i] = 0;
        }
        board.insert_loc(hex(0, 0), Bug::Spider, Color::White); // r = 0
        board.insert_loc(hex(1, 0), Bug::Spider, Color::Black); // r = -1
        let mut turns = Vec::new();
        board.generate_placements(&mut turns);
        board.assert_placements(
            &turns,
            &[
                (hex(0, -1), Bug::Queen),
                (hex(-1, 0), Bug::Queen),
                (hex(-1, 1), Bug::Queen),
            ],
        );
    }

    #[test]
    fn test_cut_vertex() {
        let mut board = Board::default();
        //. . Q Q Q Q
        // . . . Q . Q Q
        //．．．．Q Q
        board.fill_board(
            &[
                hex(0, 0),
                hex(0, 1),
                hex(1, -1),
                hex(0, -1),
                hex(-1, -1),
                hex(2, -1),
                hex(1, 1),
                hex(2, 0),
                hex(3, 0),
            ],
            Bug::Queen,
        );
        let cuts = board.find_cut_vertices();
        let mut cut_locs = vec![];
        for (hex, _) in board.nodes.iter() {
            if cuts.contains(board.find_id(hex)) {
                cut_locs.push(hex);
            }
        }
        let cut_locs_set: HashSet<Hex> = cut_locs.iter().copied().collect();
        assert_eq!(&HashSet::from([hex(0, -1), hex(2, 0)]), &cut_locs_set);
    }

    #[test]
    fn test_slideable() {
        let mut board = Board::default();
        let x = START_HEX;
        let mut buf = [Hex::ZERO; 6];
        // One neighbor.
        board.insert_loc(hex(0, 0), Bug::Queen, Color::Black);
        board.insert_loc(hex(1, 0), Bug::Queen, Color::Black);
        assert_eq!(
            vec![hex(1, -1), hex(0, 1)]
                .into_iter()
                .collect::<HashSet<Hex>>(),
            board
                .slideable_adjacent(&mut buf, x, x)
                .collect::<HashSet<Hex>>()
        );
        // Two adjacent neighbors.
        board.insert_loc(hex(0, 1), Bug::Queen, Color::Black);
        assert_eq!(
            vec![hex(1, -1), hex(-1, 1)]
                .into_iter()
                .collect::<HashSet<Hex>>(),
            board
                .slideable_adjacent(&mut buf, x, x)
                .collect::<HashSet<Hex>>()
        );
        // Four adjacent neighbors.
        board.insert_loc(hex(-1, 1), Bug::Queen, Color::Black);
        board.insert_loc(hex(1, -1), Bug::Queen, Color::Black);
        assert_eq!(
            vec![hex(-1, 0), hex(0, -1)]
                .into_iter()
                .collect::<HashSet<Hex>>(),
            board
                .slideable_adjacent(&mut buf, x, x)
                .collect::<HashSet<Hex>>()
        );
        // Five adjacent neighbors.
        board.insert_loc(hex(-1, 0), Bug::Queen, Color::Black);
        assert_eq!(
            HashSet::<Hex>::new(),
            board
                .slideable_adjacent(&mut buf, x, x)
                .collect::<HashSet<Hex>>()
        );
        // 2 separated groups of neighbors.
        board.remove_loc(hex(0, 1));
        assert_eq!(
            HashSet::<Hex>::new(),
            board
                .slideable_adjacent(&mut buf, x, x)
                .collect::<HashSet<Hex>>()
        );
        // 2 opposite single neighbors
        board.remove_loc(hex(-1, 1));
        board.remove_loc(hex(1, -1));
        println!("{:?}", board.occupied_hexes);
        assert_eq!(
            vec![hex(0, -1), hex(1, -1), hex(-1, 1), hex(0, 1)]
                .into_iter()
                .collect::<HashSet<Hex>>(),
            board
                .slideable_adjacent(&mut buf, x, x)
                .collect::<HashSet<Hex>>()
        );
    }

    #[test]
    fn test_generate_jumps() {
        let mut board = Board::default();
        //. . . g g g ．
        // ．．g ．．．
        //．．．．．．
        // ．g ．．
        board.fill_board(
            &[hex(0, 0), hex(1, 0), hex(2, 0), hex(-1, 1), hex(-3, 3)],
            Bug::Grasshopper,
        );
        let mut turns = Vec::new();
        board.generate_jumps(START_HEX, &mut turns);
        board.assert_movements(&turns, hex(0, 0), &[hex(-2, 2), hex(3, 0)]);
    }

    #[test]
    fn test_generate_beetle() {
        let mut board = Board::default();
        board.fill_board(
            &[
                hex(0, 0),
                hex(0, 0),
                hex(1, -1),
                hex(1, -1),
                hex(1, -1),
                hex(1, 0),
                hex(1, 0),
                hex(0, 1),
                hex(0, 1),
                hex(0, 1),
                hex(-1, 1),
                hex(-1, 1),
                hex(0, -1),
                hex(0, -1),
            ],
            Bug::Beetle,
        );
        // Stack heights:
        //   2   3
        //  0 (2) 2
        //   2   3
        // Can't move left (down) or right (up) because of blocking stacks.
        // Can move onto all 4 blocking stacks.
        let mut turns = Vec::new();
        board.generate_stack_walking(START_HEX, &mut turns);
        board.assert_movements(
            &turns,
            hex(0, 0),
            &[hex(0, -1), hex(1, -1), hex(-1, 1), hex(0, 1)],
        );
    }

    #[test]
    fn test_generate_walk3() {
        let mut board = Board::default();
        //．．．🕷．．．．．
        // ．．．🕷．🕷．．
        //．．．🕷．．🕷．
        // ．．．🕷🕷🕷
        board.fill_board(
            &[
                hex(0, -1),
                hex(0, 0),
                hex(2, 0),
                hex(-1, 1),
                hex(-1, 2),
                hex(0, 2),
                hex(1, 2),
                hex(2, 1),
            ],
            Bug::Spider,
        );
        let mut turns = Vec::new();
        let start = hex(0, -1);
        board.generate_walk3(start, &mut turns);
        board.assert_movements(
            &turns,
            hex(0, -1),
            &[hex(0, 1), hex(2, -1), hex(-2, 2), hex(1, 1)],
        );

        // ．．🕷．🕷．．
        //．．🕷🕷．🕷．
        // ．．🕷🕷🕷
        board.remove_loc(hex(0, -1));
        board.insert_loc(hex(0, 1), Bug::Spider, Color::Black);
        turns.clear();
        let start = hex(0, 1);
        board.generate_walk3(start, &mut turns);
        board.assert_movements(
            &turns,
            hex(0, 1),
            &[hex(0, -1), hex(2, -1), hex(1, -1), hex(3, -1)],
        );
    }

    #[test]
    fn test_generate_walk_all() {
        let mut board = Board::default();
        //．．．🐜．．．．
        // ．．．🐜．．．
        //．．．🐜．🐜．
        // ．．．🐜🐜
        board.fill_board(
            &[
                hex(0, -1),
                hex(0, 0),
                hex(-1, 1),
                hex(-1, 2),
                hex(0, 2),
                hex(1, 1),
            ],
            Bug::Ant,
        );
        let mut turns = Vec::new();
        let start = hex(0, -1);
        board.generate_walk_all(start, &mut turns);
        board.assert_movements(
            &turns,
            hex(0, -1),
            &[
                hex(-1, 0),
                hex(-2, 1),
                hex(-2, 2),
                hex(-2, 3),
                hex(-1, 3),
                hex(0, 3),
                hex(1, 2),
                hex(2, 1),
                hex(2, 0),
                hex(1, 0),
                hex(1, -1),
            ],
        );
    }

    #[test]
    fn test_generate_mosquito() {
        let mut board = Board::default();
        board.fill_board(&[hex(0, 0), hex(-1, 1)], Bug::Mosquito);
        let mut turns = Vec::new();
        board.generate_mosquito(hex(0, 0), &mut turns);
        // Mosquito on mosquito can't move at all.
        board.assert_movements(&turns, hex(0, 0), &[]);

        //．．🦟🦗．
        // ．🐜🪲．
        board.insert_loc(hex(-1, 1), Bug::Ant, Color::Black);
        board.insert_loc(hex(0, 1), Bug::Beetle, Color::Black);
        board.insert_loc(hex(1, 0), Bug::Grasshopper, Color::Black);
        turns.clear();
        // Dedup happens in generate_movements.
        board.turn_num += 1;
        board.generate_movements(&mut turns);
        board.assert_movements(
            &turns,
            hex(0, 0),
            &[
                hex(1, -1),
                hex(2, -1),
                hex(-1, 0),
                hex(1, 0),
                hex(2, 0),
                hex(-2, 1),
                hex(-1, 1),
                hex(0, 1),
                hex(1, 1),
                hex(-2, 2),
                hex(-1, 2),
                hex(0, 2),
            ],
        );
    }

    #[test]
    fn test_generate_ladybug() {
        let mut board = Board::default();
        board.fill_board(
            &[
                hex(0, 0),
                hex(-1, 1),
                hex(1, 1),
                hex(-1, 2),
                hex(0, 2),
                hex(-1, 3),
            ],
            Bug::Ladybug,
        );
        //．．．🐞．．．
        // ．．🐞．🐞．．
        //．．．🐞🐞．．
        // ．．．🐞．．
        let mut turns = Vec::new();
        let start = hex(-1, 3);
        board.generate_ladybug(start, &mut turns);
        board.assert_movements(
            &turns,
            hex(-1, 3),
            &[
                hex(-2, 3),
                hex(0, 3),
                hex(-2, 2),
                hex(1, 2),
                hex(0, 1),
                hex(2, 1),
                hex(-2, 1),
                hex(-1, 0),
                hex(1, 0),
                hex(2, 0),
            ],
        );
    }

    #[test]
    fn test_generate_throws() {
        let mut board = Board::default();
        board.fill_board(
            &[
                hex(0, 0),
                hex(0, 0),
                hex(-1, 1),
                hex(0, 1),
                hex(-1, 2),
                hex(0, 2),
                hex(0, 2),
            ],
            Bug::Pillbug,
        );
        // ．．💊．．．
        //．．💊💊．．
        // ．．💊💊．
        let mut turns = Vec::new();
        let immovable = HexSet::new();
        let mut starts = HexSet::new();
        let mut ends = HexSet::new();
        let start = hex(0, 1);
        board.generate_throws(&immovable, start, &mut turns, &mut starts, &mut ends);
        assert_eq!(4, turns.len());
        board.assert_movements(&turns, hex(-1, 2), &[hex(1, 0), hex(1, 1)]);
        board.assert_movements(&turns, hex(-1, 1), &[hex(1, 0), hex(1, 1)]);

        // Create a level-2 gate to prevent one piece from being thrown.
        board.remove_loc(hex(0, 0));
        board.insert_loc(hex(-1, 1), Bug::Pillbug, Color::Black);
        turns.clear();
        board.generate_throws(&immovable, start, &mut turns, &mut starts, &mut ends);
        assert_eq!(2, turns.len());
        board.assert_movements(&turns, hex(0, 0), &[hex(1, 0), hex(1, 1)]);

        // Create a level-2 gate to prevent one destination from being thrown to.
        board.insert_loc(hex(1, 0), Bug::Pillbug, Color::Black);
        board.insert_loc(hex(1, 0), Bug::Pillbug, Color::Black);
        board.remove_loc(hex(-1, 1));
        board.remove_loc(hex(-1, 1));
        board.remove_loc(hex(-1, 2));
        turns = Vec::new();
        board.generate_throws(&immovable, start, &mut turns, &mut starts, &mut ends);
        assert_eq!(2, turns.len());
        board.assert_movements(&turns, hex(0, 0), &[hex(-1, 1), hex(-1, 2)]);
    }

    #[test]
    fn test_winner() {
        // Draw by threefold repetition.
        let mut board = Board::default();
        let x1 = hex(0, -1);
        let x2 = hex(-1, 0);
        let y1 = hex(0, 1);
        let y2 = hex(1, 0);
        board.apply(Turn::Place(hex(0, 0), Bug::Spider));
        assert_eq!(None, Rules::get_winner(&board));
        board.apply(Turn::Place(x1, Bug::Queen));
        assert_eq!(None, Rules::get_winner(&board));
        // Create the position the first time.
        board.apply(Turn::Place(y1, Bug::Queen));
        assert_eq!(None, Rules::get_winner(&board));
        board.apply(Turn::Move(x1, x2));
        assert_eq!(None, Rules::get_winner(&board));
        board.apply(Turn::Move(y1, y2));
        assert_eq!(None, Rules::get_winner(&board));
        board.apply(Turn::Move(x2, x1));
        assert_eq!(None, Rules::get_winner(&board));
        // Recreate position for the second time.
        board.apply(Turn::Move(y2, y1));
        assert_eq!(None, Rules::get_winner(&board));
        board.apply(Turn::Move(x1, x2));
        assert_eq!(None, Rules::get_winner(&board));
        board.apply(Turn::Move(y1, y2));
        assert_eq!(None, Rules::get_winner(&board));
        board.apply(Turn::Move(x2, x1));
        assert_eq!(None, Rules::get_winner(&board));
        // Recreate position for the third time.
        board.apply(Turn::Move(y2, y1));
        assert_eq!(Some(Winner::Draw), Rules::get_winner(&board));
        // Undo reverts zobrist and history.
        board.undo(Turn::Move(y2, y1));
        assert_eq!(None, Rules::get_winner(&board));
        // Redo re-reverts draw state.
        board.apply(Turn::Move(y2, y1));
        assert_eq!(Some(Winner::Draw), Rules::get_winner(&board));
    }
    #[test]
    fn test_winner5() {
        // Regression test for issue #5.
        let board = Board::from_game_string(r"Base;InProgress;black[11];wA1;bA1 wA1-;wQ /wA1;bQ bA1/;wS1 /wQ;bS1 bQ-;wS2 wQ\;bS2 bS1\;wS1 wS2-;bB1 /bS2;wA2 -wA1;bA2 bA1-;wB1 \wA1;bB1 bA2;wB1 wA2;bB1 bQ;wB1 wQ;bB1 \bB1;wB1 wB1-;bB1 bQ;wB1 wQ").unwrap();
        assert_eq!(None, Rules::get_winner(&board));
    }
}
