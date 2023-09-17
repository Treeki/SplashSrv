pub mod appearance;
pub mod character;
pub mod item;
pub mod record;
pub mod report;
pub mod shop;

pub use appearance::Appearance;
pub use character::{CharID, Character, ParamTuple};
pub use item::{CountedItem, Item, ItemCategory};
pub use shop::{SellCaddy, SellItem};

use crate::data::shop::Currency;
use deku::prelude::*;
use serde::{Deserialize, Serialize};

use crate::packets::{ChrUID, Element, UID};

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, DekuRead, DekuWrite,
)]
#[deku(type = "i8")]
pub enum Class {
    G = 0,
    F = 1,
    E = 2,
    D = 3,
    C = 4,
    B = 5,
    A = 6,
    S = 7,
}

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, DekuRead, DekuWrite,
)]
#[deku(type = "i8")]
pub enum Rank {
    G4 = 0,
    G3 = 1,
    G2 = 2,
    G1 = 3,
    F4 = 4,
    F3 = 5,
    F2 = 6,
    F1 = 7,
    E4 = 8,
    E3 = 9,
    E2 = 10,
    E1 = 11,
    D4 = 12,
    D3 = 13,
    D2 = 14,
    D1 = 15,
    C4 = 16,
    C3 = 17,
    C2 = 18,
    C1 = 19,
    B4 = 20,
    B3 = 21,
    B2 = 22,
    B1 = 23,
    A4 = 24,
    A3 = 25,
    A2 = 26,
    A1 = 27,
    S4 = 28,
    S3 = 29,
    S2 = 30,
    S1 = 31,
}

impl Rank {
    pub fn class(self) -> Class {
        use Rank::*;
        match self {
            G4 | G3 | G2 | G1 => Class::G,
            F4 | F3 | F2 | F1 => Class::F,
            E4 | E3 | E2 | E1 => Class::E,
            D4 | D3 | D2 | D1 => Class::D,
            C4 | C3 | C2 | C1 => Class::C,
            B4 | B3 | B2 | B1 => Class::B,
            A4 | A3 | A2 | A1 => Class::A,
            S4 | S3 | S2 | S1 => Class::S,
        }
    }
}

#[derive(Debug)]
pub struct Account {
    pub uid: UID,
    pub name: Option<String>,
    pub user: User,
    pub characters: Vec<(ChrUID, Character)>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub default_chr_uid: ChrUID,
    pub element: Element,
    pub class: Rank,
    pub gp: i32,
    pub sc: i32,
    pub golfbag: [Item; 8],
    pub holdbox: [Item; 8],
    pub inventory: Vec<CountedItem>,
}

impl Default for User {
    fn default() -> Self {
        User {
            default_chr_uid: -1,
            element: Element::None,
            class: Rank::G4,
            gp: 5000,
            sc: 100,
            golfbag: Default::default(),
            holdbox: Default::default(),
            inventory: Vec::new(),
        }
    }
}

impl User {
    /// Get the quantity of a particular item within the user's inventory
    pub fn item_amount(&self, item: Item) -> u32 {
        match self.inventory.iter().find(|ci| ci.item() == item) {
            Some(ci) => ci.count(),
            None => 0,
        }
    }

    /// Add a particular item to the user's inventory
    pub fn add_item(&mut self, counted_item: CountedItem) {
        let item = counted_item.item();
        let count = counted_item.count();

        match self.inventory.iter_mut().find(|ci| ci.item() == item) {
            Some(ci) => *ci = ci.with_count(ci.count() + count),
            None => self.inventory.push(counted_item),
        }
    }

    /// Check if the user has enough money to buy something
    pub fn check_balance(&self, currency: Currency, cost: i32) -> bool {
        match currency {
            Currency::GP => cost <= self.gp,
            Currency::SC => cost <= self.sc,
            Currency::TicketsOnly => false,
        }
    }

    /// Update the user's balance by adding or subtracting money
    pub fn adjust_balance(&mut self, currency: Currency, delta: i32) {
        match currency {
            Currency::GP => self.gp += delta,
            Currency::SC => self.sc += delta,
            Currency::TicketsOnly => panic!("invalid"),
        }
    }
}
