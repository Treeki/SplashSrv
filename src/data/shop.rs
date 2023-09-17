use super::{CharID, Item, ItemCategory};
use deku::bitvec::{BitSlice, BitVec, Msb0};
use deku::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Currency {
    GP,
    SC,
    TicketsOnly,
}

impl Currency {
    fn from_flags(flags: u32) -> Self {
        if (flags & 0x20) != 0 {
            Currency::TicketsOnly
        } else if (flags & 2) != 0 {
            Currency::SC
        } else {
            Currency::GP
        }
    }

    fn to_flags(self) -> u32 {
        match self {
            Self::GP => 0,
            Self::SC => 2,
            Self::TicketsOnly => 0x20,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Marketing {
    None,
    New,
    Hot,
    Sale,
}

impl Marketing {
    fn from_flags(flags: u32) -> Self {
        if (flags & 0x40) != 0 {
            Marketing::Sale
        } else if (flags & 0x10) != 0 {
            Marketing::Hot
        } else if (flags & 8) != 0 {
            Marketing::Hot
        } else {
            Marketing::None
        }
    }

    fn to_flags(self) -> u32 {
        match self {
            Self::None => 0,
            Self::New => 8,
            Self::Hot => 0x10,
            Self::Sale => 0x40,
        }
    }
}

// Configuration for items that are available in the shops
#[derive(Debug, Clone)]
pub struct SellItem {
    pub item: Item,
    pub currency: Currency,
    pub marketing: Marketing,
    pub price: u32,
    pub sp_price: u32,
}

impl DekuRead<'_> for SellItem {
    fn read(input: &BitSlice<u8, Msb0>, ctx: ()) -> Result<(&BitSlice<u8, Msb0>, Self), DekuError>
    where
        Self: Sized,
    {
        // Offset 0
        let (input, val) = u32::read(input, ctx)?;
        let item = Item(val);

        // Offset 4
        let (input, val) = u32::read(input, ctx)?;
        let price = val & 0xFFFFF;
        let flags = val >> 20;
        let currency = Currency::from_flags(flags);
        let marketing = Marketing::from_flags(flags);

        // Offset 8
        let (input, val) = u32::read(input, ctx)?;
        let sp_price = val & 0xFFFFF;

        let sell = SellItem {
            item,
            currency,
            marketing,
            price,
            sp_price,
        };
        Ok((input, sell))
    }
}

impl DekuWrite for SellItem {
    fn write(&self, output: &mut BitVec<u8, Msb0>, ctx: ()) -> Result<(), DekuError> {
        // Offset 0
        let val: u32 = self.item.0;
        val.write(output, ctx)?;

        // Offset 4
        let flags = self.currency.to_flags() | self.marketing.to_flags();
        let val: u32 = (self.price & 0xFFFFF) | (flags << 20);
        val.write(output, ctx)?;

        // Offset 8
        let val: u32 = self.sp_price & 0xFFFFF;
        val.write(output, ctx)?;

        Ok(())
    }
}

// Configuration for a Caddy that can be rented
#[derive(Debug, Clone)]
pub struct SellCaddy {
    pub item: Item,
    pub currency: Currency,
    pub marketing: Marketing,
    pub price_3_hours: u32,
    pub price_3_days: u32,
    pub price_30_days: u32,
    pub infinite_rental: i32,
}

impl DekuRead<'_> for SellCaddy {
    fn read(input: &BitSlice<u8, Msb0>, ctx: ()) -> Result<(&BitSlice<u8, Msb0>, Self), DekuError>
    where
        Self: Sized,
    {
        let (input, val) = u32::read(input, ctx)?;
        let item = Item(val & 0x3FFFFF);
        let flags = val >> 22;
        let currency = Currency::from_flags(flags);
        let marketing = Marketing::from_flags(flags);

        let (input, price_3_hours) = u32::read(input, ctx)?;
        let (input, price_3_days) = u32::read(input, ctx)?;
        let (input, price_30_days) = u32::read(input, ctx)?;
        let (input, infinite_rental) = i32::read(input, ctx)?;

        let sell = SellCaddy {
            item,
            currency,
            marketing,
            price_3_hours,
            price_3_days,
            price_30_days,
            infinite_rental,
        };
        Ok((input, sell))
    }
}

impl DekuWrite for SellCaddy {
    fn write(&self, output: &mut BitVec<u8, Msb0>, ctx: ()) -> Result<(), DekuError> {
        let flags = self.currency.to_flags() | self.marketing.to_flags();
        let val: u32 = self.item.0 | (flags << 22);
        val.write(output, ctx)?;

        self.price_3_hours.write(output, ctx)?;
        self.price_3_days.write(output, ctx)?;
        self.price_30_days.write(output, ctx)?;
        self.infinite_rental.write(output, ctx)?;

        Ok(())
    }
}

pub fn build_sell_list() -> Vec<SellItem> {
    let mut list = Vec::new();

    let currency = Currency::GP;
    let marketing = Marketing::None;

    let types = [
        (ItemCategory::ClubSet, 1..=55),
        (ItemCategory::Ball, 1..=15),
        (ItemCategory::CarryItemEnvironment, 1..=12),
        (ItemCategory::CarryItemGroundRes, 1..=18),
        (ItemCategory::CarryItemPowerGauge, 1..=6),
        (ItemCategory::CarryItemCaddy, 1..=10),
        (ItemCategory::HoldItemPoint, 1..=6),
        (ItemCategory::HoldItemEvent, 1..=3),
        // this doesn't count character-specific tickets...
        (ItemCategory::HoldItemTicket, 1..=41),
        (ItemCategory::HoldItemHumor, 1..=3),
        (ItemCategory::HoldItemSupport, 1..=7),
    ];

    for (category, range) in types {
        for num in range {
            let item = Item::new(category, num);
            list.push(SellItem {
                item,
                currency,
                marketing,
                price: num * 5,
                sp_price: 0,
            });
        }
    }

    // parameter items
    for group in 0..5 {
        let indices = [1, 3, 4, 6];
        for i in 0..4 {
            let item = Item::new(ItemCategory::CarryItemParameter, (group * 6) + indices[i]);
            let price = 5 + (i as u32) * 10;
            list.push(SellItem {
                item,
                currency,
                marketing,
                price,
                sp_price: 0,
            });
        }
    }
    for group in 0..5 {
        for i in 0..2 {
            let item = Item::new(ItemCategory::CarryItemParameter, 31 + (group * 2) + i);
            let price = 20 + 10 * i;
            list.push(SellItem {
                item,
                currency,
                marketing,
                price,
                sp_price: 0,
            });
        }
    }

    // Add everything that Rusk can wear
    let tops = [1..=54, 996..=999];
    for range in tops {
        for num in range {
            let item = Item::new(ItemCategory::Tops(CharID::Rusk), num);
            let price = 10 * num;
            list.push(SellItem {
                item,
                currency,
                marketing,
                price,
                sp_price: 0,
            });
        }
    }

    let bottoms = [1..=25, 28..=72, 996..=999];
    for range in bottoms {
        for num in range {
            let item = Item::new(ItemCategory::Bottoms(CharID::Rusk), num);
            let price = 10 * num;
            list.push(SellItem {
                item,
                currency,
                marketing,
                price,
                sp_price: 0,
            });
        }
    }

    let shoes = [1..=43, 996..=999];
    for range in shoes {
        for num in range {
            let item = Item::new(ItemCategory::Shoes(CharID::Rusk), num);
            let price = 10 * num;
            list.push(SellItem {
                item,
                currency,
                marketing,
                price,
                sp_price: 0,
            });
        }
    }

    for num in 1..=22 {
        let item = Item::new(ItemCategory::Head(CharID::Rusk), num);
        let price = 10 * num;
        list.push(SellItem {
            item,
            currency,
            marketing,
            price,
            sp_price: 0,
        });
    }
    for num in 1..=11 {
        if num != 3 {
            let item = Item::new(ItemCategory::Glasses(CharID::Rusk), num);
            let price = 10 * num;
            list.push(SellItem {
                item,
                currency,
                marketing,
                price,
                sp_price: 0,
            });
        }
    }
    for num in 1..=9 {
        let item = Item::new(ItemCategory::Gloves(CharID::Rusk), num);
        let price = 10 * num;
        list.push(SellItem {
            item,
            currency,
            marketing,
            price,
            sp_price: 0,
        });
    }
    for num in 1..=7 {
        let item = Item::new(ItemCategory::Wing(CharID::Rusk), num);
        let price = 10 * num;
        list.push(SellItem {
            item,
            currency,
            marketing,
            price,
            sp_price: 0,
        });
    }

    // Add everything that Miel can wear
    let tops = [1..=5, 7..=62, 996..=999];
    for range in tops {
        for num in range {
            let item = Item::new(ItemCategory::Tops(CharID::Miel), num);
            let price = 10 * num;
            list.push(SellItem {
                item,
                currency,
                marketing,
                price,
                sp_price: 0,
            });
        }
    }

    let bottoms = [2..=11, 13..=13, 15..=34, 36..=45, 47..=60, 996..=999];
    for range in bottoms {
        for num in range {
            let item = Item::new(ItemCategory::Bottoms(CharID::Miel), num);
            let price = 10 * num;
            list.push(SellItem {
                item,
                currency,
                marketing,
                price,
                sp_price: 0,
            });
        }
    }

    let shoes = [1..=47, 996..=999];
    for range in shoes {
        for num in range {
            let item = Item::new(ItemCategory::Shoes(CharID::Miel), num);
            let price = 10 * num;
            list.push(SellItem {
                item,
                currency,
                marketing,
                price,
                sp_price: 0,
            });
        }
    }

    for num in 1..=22 {
        let item = Item::new(ItemCategory::Head(CharID::Miel), num);
        let price = 10 * num;
        list.push(SellItem {
            item,
            currency,
            marketing,
            price,
            sp_price: 0,
        });
    }
    for num in 1..=12 {
        if num != 3 {
            let item = Item::new(ItemCategory::Glasses(CharID::Miel), num);
            let price = 10 * num;
            list.push(SellItem {
                item,
                currency,
                marketing,
                price,
                sp_price: 0,
            });
        }
    }
    for num in 1..=10 {
        let item = Item::new(ItemCategory::Gloves(CharID::Miel), num);
        let price = 10 * num;
        list.push(SellItem {
            item,
            currency,
            marketing,
            price,
            sp_price: 0,
        });
    }
    for num in 1..=4 {
        let item = Item::new(ItemCategory::Wing(CharID::Miel), num);
        let price = 10 * num;
        list.push(SellItem {
            item,
            currency,
            marketing,
            price,
            sp_price: 0,
        });
    }

    // Add everything that Gouda can wear
    let tops = [1..=60];
    for range in tops {
        for num in range {
            let item = Item::new(ItemCategory::Tops(CharID::Gouda), num);
            let price = 10 * num;
            list.push(SellItem {
                item,
                currency,
                marketing,
                price,
                sp_price: 0,
            });
        }
    }

    let bottoms = [1..=69];
    for range in bottoms {
        for num in range {
            let item = Item::new(ItemCategory::Bottoms(CharID::Gouda), num);
            let price = 10 * num;
            list.push(SellItem {
                item,
                currency,
                marketing,
                price,
                sp_price: 0,
            });
        }
    }

    let shoes = [1..=51];
    for range in shoes {
        for num in range {
            let item = Item::new(ItemCategory::Shoes(CharID::Gouda), num);
            let price = 10 * num;
            list.push(SellItem {
                item,
                currency,
                marketing,
                price,
                sp_price: 0,
            });
        }
    }

    for num in 1..=18 {
        let item = Item::new(ItemCategory::Head(CharID::Gouda), num);
        let price = 10 * num;
        list.push(SellItem {
            item,
            currency,
            marketing,
            price,
            sp_price: 0,
        });
    }
    for num in 1..=13 {
        if num != 3 && num != 5 && num != 7 && num != 10 && num != 11 {
            let item = Item::new(ItemCategory::Glasses(CharID::Gouda), num);
            let price = 10 * num;
            list.push(SellItem {
                item,
                currency,
                marketing,
                price,
                sp_price: 0,
            });
        }
    }
    for num in 1..=11 {
        let item = Item::new(ItemCategory::Gloves(CharID::Gouda), num);
        let price = 10 * num;
        list.push(SellItem {
            item,
            currency,
            marketing,
            price,
            sp_price: 0,
        });
    }
    for num in 1..=6 {
        let item = Item::new(ItemCategory::Wing(CharID::Gouda), num);
        let price = 10 * num;
        list.push(SellItem {
            item,
            currency,
            marketing,
            price,
            sp_price: 0,
        });
    }

    list
}

pub fn build_salon_list() -> Vec<SellItem> {
    let mut list = Vec::new();
    let chars = [
        (CharID::Rusk, 15),
        (CharID::Miel, 18),
        (CharID::Rose, 19),
        (CharID::Chocola, 18),
        (CharID::Shelly, 18),
        (CharID::Gouda, 20),
        (CharID::Sect, 16),
    ];

    for (c, num_face_paints) in chars {
        // Each character has 4 hair styles, 4 hair colours and 4 skin colours
        for num in 1..=4 {
            list.push(SellItem {
                item: Item::new(ItemCategory::HairStyle(c), num),
                currency: Currency::GP,
                marketing: Marketing::None,
                price: 15 * num,
                sp_price: 0,
            });

            list.push(SellItem {
                item: Item::new(ItemCategory::HairColor(c), num),
                currency: Currency::GP,
                marketing: Marketing::None,
                price: 5 * num,
                sp_price: 0,
            });

            list.push(SellItem {
                item: Item::new(ItemCategory::SkinColor(c), num),
                currency: Currency::GP,
                marketing: Marketing::None,
                price: 25 * num,
                sp_price: 0,
            });
        }

        // Each character has 10 eye colours
        for num in 1..=10 {
            list.push(SellItem {
                item: Item::new(ItemCategory::EyeColor(c), num),
                currency: Currency::GP,
                marketing: Marketing::None,
                price: 20 * num,
                sp_price: 0,
            });
        }

        // Number of face paints is variable
        for num in 1..=num_face_paints {
            list.push(SellItem {
                item: Item::new(ItemCategory::FacePaint(c), num),
                currency: Currency::GP,
                marketing: Marketing::None,
                price: 3 * num,
                sp_price: 0,
            });
        }
    }

    list
}
