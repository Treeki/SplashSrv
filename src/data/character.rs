use deku::prelude::*;
use serde::{Deserialize, Serialize};

use crate::packets::{ChrData, ChrUID};

use super::{Appearance, Class, Item, ItemCategory};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum CharID {
    Rusk,
    Miel,
    Rose,
    Chocola,
    Shelly,
    Gouda,
    Sect,
}

impl CharID {
    pub fn to_index(self) -> u32 {
        match self {
            Self::Rusk => 1,
            Self::Miel => 2,
            Self::Rose => 3,
            Self::Chocola => 4,
            Self::Shelly => 5,
            Self::Gouda => 6,
            Self::Sect => 7,
        }
    }

    pub fn from_index(index: u32) -> Option<Self> {
        match index {
            1 => Some(Self::Rusk),
            2 => Some(Self::Miel),
            3 => Some(Self::Rose),
            4 => Some(Self::Chocola),
            5 => Some(Self::Shelly),
            6 => Some(Self::Gouda),
            7 => Some(Self::Sect),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, DekuRead, DekuWrite)]
pub struct ParamTuple {
    pub power: i16,
    pub control: i16, // these two might be swapped?
    pub impact: i16,  // ...^^
    pub spin: i16,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Character {
    // Denotes which entry in 'settings' this character is currently using
    pub class_cap: Class,
    // Denotes the experience this character has in each parameter
    pub exp: ParamTuple,
    // Denotes how points are allocated for each class
    pub settings: [ParamTuple; 8],

    pub appearance: Appearance,
    pub club: Item,
    pub ball: Item,
    pub caddie: Item,
}

impl Character {
    // create the initial character
    pub fn new(appearance: Appearance) -> Self {
        Character {
            class_cap: Class::G,
            exp: ParamTuple {
                power: 0,
                control: 0,
                impact: 0,
                spin: 0,
            },
            settings: [ParamTuple {
                power: 0,
                control: 0,
                impact: 0,
                spin: 0,
            }; 8],
            appearance,
            club: Item::new(ItemCategory::ClubSet, 2),
            ball: Item::new(ItemCategory::Ball, 1),
            caddie: Item::default(),
        }
    }

    pub fn to_chr_data(&self, chr_uid: ChrUID) -> ChrData {
        ChrData {
            chr_uid,
            type_: self.appearance.character_id.to_index() as i16,
            class: self.class_cap,
            x_7: 0,
            param_power: self.exp.power,
            param_control: self.exp.control,
            param_impact: self.exp.impact,
            param_spin: self.exp.spin,
            x_10: [0; 16], // ???
            param_settings: self.settings,
            appearance: self.appearance.clone(),
            club: self.club,
            ball: self.ball,
            caddie: self.caddie,
        }
    }
}
