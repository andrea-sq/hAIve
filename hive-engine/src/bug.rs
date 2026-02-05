use bitfield::BitRange;
use color_eyre::eyre::eyre;
use color_eyre::Report;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[repr(u8)]
pub enum Bug {
    Queen = 0,
    Grasshopper = 1,
    Spider = 2,
    Ant = 3,
    Beetle = 4
}

impl Bug {
    pub fn name(&self) -> &'static str {
        match self {
            Bug::Queen => "queen",
            Bug::Grasshopper => "grasshopper",
            Bug::Spider => "spider",
            Bug::Ant => "ant",
            Bug::Beetle => "beetle",
        }
    }

    pub(crate) fn initial_quantity() -> &'static [u8; 5] {
        &[1, 3, 2, 3, 2]
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
            _ => Err(eyre!("Invalid bug id: {}", value)),
        }
    }
}

impl Into<u8> for Bug {
    fn into(self) -> u8 {
        self as u8
    }
}



