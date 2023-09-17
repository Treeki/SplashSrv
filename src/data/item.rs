use super::CharID;
use deku::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ItemCategory {
    ClubSet,
    Ball,
    CarryItemParameter,
    CarryItemEnvironment,
    CarryItemGroundRes,
    CarryItemPowerGauge,
    CarryItemCaddy,
    HoldItemPoint,
    HoldItemEvent,
    HoldItemTicket,
    HoldItemHumor,
    HoldItemSupport,
    Caddy,
    Head(CharID),
    Glasses(CharID),
    Tops(CharID),
    Bottoms(CharID),
    Shoes(CharID),
    Gloves(CharID),
    Wing(CharID),
    HairStyle(CharID),
    HairColor(CharID),
    SkinColor(CharID),
    FacePaint(CharID),
    EyeColor(CharID),
    HairStyleTicket(CharID),
    HairColorTicket(CharID),
    SkinColorTicket(CharID),
    FacePaintTicket(CharID),
    EyeColorTicket(CharID),
    Chara(CharID),
    Invalid,
}

impl ItemCategory {
    pub fn character(self) -> Option<CharID> {
        use ItemCategory::*;

        match self {
            ClubSet => None,
            Ball => None,
            CarryItemParameter => None,
            CarryItemEnvironment => None,
            CarryItemGroundRes => None,
            CarryItemPowerGauge => None,
            CarryItemCaddy => None,
            HoldItemPoint => None,
            HoldItemEvent => None,
            HoldItemTicket => None,
            HoldItemHumor => None,
            HoldItemSupport => None,
            Caddy => None,
            Head(c) => Some(c),
            Glasses(c) => Some(c),
            Tops(c) => Some(c),
            Bottoms(c) => Some(c),
            Shoes(c) => Some(c),
            Gloves(c) => Some(c),
            Wing(c) => Some(c),
            HairStyle(c) => Some(c),
            HairColor(c) => Some(c),
            SkinColor(c) => Some(c),
            FacePaint(c) => Some(c),
            EyeColor(c) => Some(c),
            HairStyleTicket(c) => Some(c),
            HairColorTicket(c) => Some(c),
            SkinColorTicket(c) => Some(c),
            FacePaintTicket(c) => Some(c),
            EyeColorTicket(c) => Some(c),
            Chara(c) => Some(c),
            Invalid => None,
        }
    }

    pub fn maximum(self) -> u32 {
        use ItemCategory::*;

        match self {
            ClubSet => 5,
            Ball => 50,
            CarryItemParameter => 50,
            CarryItemEnvironment => 50,
            CarryItemGroundRes => 50,
            CarryItemPowerGauge => 50,
            CarryItemCaddy => 50,
            HoldItemPoint => 50,
            HoldItemEvent => 50,
            HoldItemTicket => 50,
            HoldItemHumor => 50,
            HoldItemSupport => 50,
            Caddy => 5,
            Head(_) => 5,
            Glasses(_) => 5,
            Tops(_) => 5,
            Bottoms(_) => 5,
            Shoes(_) => 5,
            Gloves(_) => 5,
            Wing(_) => 5,
            HairStyle(_) => 5,
            HairColor(_) => 5,
            SkinColor(_) => 5,
            FacePaint(_) => 5,
            EyeColor(_) => 5,
            HairStyleTicket(_) => 50,
            HairColorTicket(_) => 50,
            SkinColorTicket(_) => 50,
            FacePaintTicket(_) => 50,
            EyeColorTicket(_) => 50,
            Chara(_) => 5,
            Invalid => 0,
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq, DekuRead, DekuWrite, Serialize, Deserialize)]
pub struct Item(pub u32);

impl Default for Item {
    fn default() -> Self {
        Item(0)
    }
}

impl Item {
    pub fn new(category: ItemCategory, num: u32) -> Self {
        use ItemCategory::*;

        let code: u32 = match category {
            ClubSet => 0x800,
            Ball => 0x3000,
            CarryItemParameter => 0x4000,
            CarryItemEnvironment => 0x5000,
            CarryItemGroundRes => 0x6000,
            CarryItemPowerGauge => 0x7000,
            CarryItemCaddy => 0x8000,
            HoldItemPoint => 0x9000,
            HoldItemEvent => 0xC000,
            HoldItemTicket => 0xD000,
            HoldItemHumor => 0xE000,
            HoldItemSupport => 0x10000,
            Caddy => 0x1F800,
            Head(c) => 0x1800 | (c.to_index() << 17),
            Glasses(c) => 0x2800 | (c.to_index() << 17),
            Tops(c) => 0x3800 | (c.to_index() << 17),
            Bottoms(c) => 0x4800 | (c.to_index() << 17),
            Shoes(c) => 0x5800 | (c.to_index() << 17),
            Gloves(c) => 0x6800 | (c.to_index() << 17),
            Wing(c) => 0x7800 | (c.to_index() << 17),
            HairStyle(c) => 0xF800 | (c.to_index() << 17),
            HairColor(c) => 0x10800 | (c.to_index() << 17),
            SkinColor(c) => 0x11800 | (c.to_index() << 17),
            FacePaint(c) => 0x12800 | (c.to_index() << 17),
            EyeColor(c) => 0x13800 | (c.to_index() << 17),
            HairStyleTicket(c) => 0x14000 | (c.to_index() << 17),
            HairColorTicket(c) => 0x15000 | (c.to_index() << 17),
            SkinColorTicket(c) => 0x16000 | (c.to_index() << 17),
            FacePaintTicket(c) => 0x17000 | (c.to_index() << 17),
            EyeColorTicket(c) => 0x18000 | (c.to_index() << 17),
            Chara(c) => 0x1F800 | (c.to_index() << 17),
            Invalid => panic!(),
        };

        assert!(num <= 0x7FF);
        Self(code | num)
    }

    pub fn one(self) -> CountedItem {
        CountedItem::new(self, 1)
    }

    pub fn num(self) -> u32 {
        self.0 & 0x7FF
    }

    pub fn category(self) -> ItemCategory {
        use ItemCategory::*;

        if self.0 == 0 {
            return Invalid;
        }

        match CharID::from_index((self.0 >> 17) & 0x1F) {
            None => match (self.0 >> 12) & 0x1F {
                0 => ClubSet,
                3 => Ball,
                4 => CarryItemParameter,
                5 => CarryItemEnvironment,
                6 => CarryItemGroundRes,
                7 => CarryItemPowerGauge,
                8 => CarryItemCaddy,
                9 => HoldItemPoint,
                0xC => HoldItemEvent,
                0xD => HoldItemTicket,
                0xE => HoldItemHumor,
                0x10 => HoldItemSupport,
                0x1F => Caddy,
                _ => Invalid,
            },
            Some(c) => match (self.0 >> 12) & 0x1F {
                1 => Head(c),
                2 => Glasses(c),
                3 => Tops(c),
                4 => Bottoms(c),
                5 => Shoes(c),
                6 => Gloves(c),
                7 => Wing(c),
                0xF => HairStyle(c),
                0x10 => HairColor(c),
                0x11 => SkinColor(c),
                0x12 => FacePaint(c),
                0x13 => EyeColor(c),
                0x14 => HairStyleTicket(c),
                0x15 => HairColorTicket(c),
                0x16 => SkinColorTicket(c),
                0x17 => FacePaintTicket(c),
                0x18 => EyeColorTicket(c),
                0x1F => Chara(c),
                _ => Invalid,
            },
        }
    }
}

impl fmt::Debug for Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 == 0 {
            write!(f, "<EmptyItem>")
        } else {
            let category = self.category();
            let num = self.num();
            write!(f, "<{category:?}:{num}>")
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq, DekuRead, DekuWrite, Serialize, Deserialize)]
pub struct CountedItem(pub u32);

impl Default for CountedItem {
    fn default() -> Self {
        CountedItem(0)
    }
}

impl CountedItem {
    pub fn new(item: Item, count: u32) -> Self {
        assert!(count <= 0x3FF);
        CountedItem((item.0 << 10) | count)
    }

    pub fn item(self) -> Item {
        Item(self.0 >> 10)
    }

    pub fn count(self) -> u32 {
        self.0 & 0x3FF
    }

    pub fn with_count(self, count: u32) -> Self {
        assert!(count <= 0x3FF);
        CountedItem((self.0 & !0x3FF) | count)
    }
}

impl fmt::Debug for CountedItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let item = self.item();
        let category = item.category();
        let num = item.num();
        let count = self.count();
        write!(f, "<{category:?}:{num} x{count}>")
    }
}
