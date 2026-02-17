use color_eyre::Report;
use color_eyre::eyre::eyre;
use modular_bitfield::prelude::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Specifier)]
#[bits = 3]
#[repr(u8)]
pub enum Bug {
    Queen = 0,
    Grasshopper = 1,
    Spider = 2,
    Ant = 3,
    Beetle = 4,
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
            Bug::Queen,
            Bug::Grasshopper,
            Bug::Spider,
            Bug::Ant,
            Bug::Beetle,
            Bug::Mosquito,
            Bug::Ladybug,
            Bug::Pillbug,
        ]
        .iter()
        .copied()
    }

    pub(crate) fn initial_quantity() -> &'static [u8; 8] {
        &[1, 3, 2, 3, 2, 1, 1, 1]
    }

    pub(crate) fn crawler(&self) -> bool {
        matches!(*self, Bug::Ant | Bug::Queen | Bug::Spider | Bug::Pillbug)
    }
}

impl TryFrom<u8> for Bug {
    type Error = Report;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Bug::Queen),
            1 => Ok(Bug::Grasshopper),
            2 => Ok(Bug::Spider),
            3 => Ok(Bug::Ant),
            4 => Ok(Bug::Beetle),
            5 => Ok(Bug::Mosquito),
            6 => Ok(Bug::Ladybug),
            7 => Ok(Bug::Pillbug),
            _ => Err(eyre!("Invalid bug id: {}", value)),
        }
    }
}

impl Into<u8> for Bug {
    fn into(self) -> u8 {
        self as u8
    }
}
