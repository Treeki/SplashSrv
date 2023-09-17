use crate::data::shop::Currency;
use crate::data::{CountedItem, Item, SellItem};
use anyhow::Result;
use log::error;

use crate::gs2::GameServer;
use crate::packets::{BuyItemResult, Packet};

impl GameServer {
    /// Return all purchasable items in regular shops to the player
    pub(super) async fn handle_get_sell_items(&self, who: usize) -> Result<()> {
        let packet = Packet::SEND_SELLITEMLIST {
            count: self.shop_items.len() as i16,
            items: self.shop_items.clone(),
        };
        self.conns[who].write(packet).await?;
        Ok(())
    }

    /// Return all purchasable items in the salon to the player
    pub(super) async fn handle_get_salon_items(&self, who: usize) -> Result<()> {
        let packet = Packet::SEND_SALON_ITEM_LIST {
            count: self.salon_items.len() as i16,
            items: self.salon_items.clone(),
        };
        self.conns[who].write(packet).await?;
        Ok(())
    }

    fn do_buy_item(&mut self, who: usize, counted_item: CountedItem) -> Result<BuyItemResult> {
        let item = counted_item.item();

        // find the corresponding metadata for this item
        let sell_item = match self.shop_items.iter().find(|s| s.item == item) {
            Some(sell_item) => sell_item.clone(),
            None => return Ok(BuyItemResult::InvalidItemType),
        };

        // enforce various checks
        let current_amount = self.conns[who].user.item_amount(item);
        if counted_item.count() <= 0 || counted_item.count() > item.category().maximum() {
            return Ok(BuyItemResult::InvalidCount);
        }

        let new_amount = current_amount + counted_item.count();
        if new_amount <= 0 || new_amount > item.category().maximum() {
            return Ok(BuyItemResult::InvalidCount);
        }

        let cost = counted_item.count() * sell_item.price;
        let cost: i32 = cost.try_into()?;
        if !self.conns[who].user.check_balance(sell_item.currency, cost) {
            return Ok(BuyItemResult::Balance);
        }

        // we should be OK
        self.conns[who]
            .user
            .adjust_balance(sell_item.currency, -cost);
        self.conns[who].user.add_item(counted_item);

        Ok(BuyItemResult::OK)
    }

    /// Try to buy a regular item using GP or SC
    pub(super) async fn handle_buy_item(&mut self, who: usize, item: CountedItem) -> Result<()> {
        let result = match self.do_buy_item(who, item) {
            Ok(r) => r,
            Err(e) => {
                error!("failed to buy item {item:?} for {who}: {e:?}");
                BuyItemResult::Err
            }
        };

        self.conns[who].write(Packet::ACK_BUY_ITEM(result)).await?;

        // update the displayed balances
        self.handle_get_money(-1, who).await?;

        self.save_user(who).await;

        Ok(())
    }
}
