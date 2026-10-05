use modular_bitfield::prelude::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Specifier)]
#[bits = 3]
#[repr(u8)]
pub enum Bug {
    Ant = 0,
    Grasshopper = 1,
    Spider = 2,
    Beetle = 3,
    Queen = 4,
    Mosquito = 5,
    Ladybug = 6,
    Pillbug = 7,
}

impl Bug {
    pub fn name(&self) -> &'static str {
        match self {
            Bug::Queen => "queen",
            Bug::Grasshopper => "grasshopper",
            Bug::Spider => "spider",
            Bug::Ant => "ant",
            Bug::Beetle => "beetle",
            Bug::Mosquito => "mosquito",
            Bug::Ladybug => "ladybug",
            Bug::Pillbug => "pillbug",
        }
    }
    pub fn to_char(&self) -> char {
        match self {
            Bug::Queen => 'q',
            Bug::Grasshopper => 'g',
            Bug::Spider => 's',
            Bug::Ant => 'a',
            Bug::Beetle => 'b',
            Bug::Mosquito => 'm',
            Bug::Ladybug => 'l',
            Bug::Pillbug => 'p',
        }
    }
    pub fn from_char(c: char) -> Option<Bug> {
        match c.to_ascii_lowercase() {
            'q' => Some(Bug::Queen),
            'g' => Some(Bug::Grasshopper),
            's' => Some(Bug::Spider),
            'a' => Some(Bug::Ant),
            'b' => Some(Bug::Beetle),
            'm' => Some(Bug::Mosquito),
            'l' => Some(Bug::Ladybug),
            'p' => Some(Bug::Pillbug),
            _ => None,
        }
    }

    pub fn iter_all() -> impl Iterator<Item = Self> {
        [
            Bug::Ant,
            Bug::Grasshopper,
            Bug::Spider,
            Bug::Beetle,
            Bug::Queen,
            Bug::Mosquito,
            Bug::Ladybug,
            Bug::Pillbug,
        ]
        .iter()
        .copied()
    }

    pub(crate) fn initial_quantity() -> &'static [u8; 8] {
        &[3, 3, 2, 2, 1, 1, 1, 1]
    }

    pub(crate) fn crawler(&self) -> bool {
        matches!(*self, Bug::Ant | Bug::Queen | Bug::Spider | Bug::Pillbug)
    }
}
