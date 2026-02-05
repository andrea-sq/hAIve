use bitfield::bitfield;
use crate::bug::Bug;

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
}


