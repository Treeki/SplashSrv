use crate::data::Item;
use anyhow::Result;
use log::{debug, error, warn};

use crate::packets::{Packet, SetPlayerName, Stat, Status, CID, UID};

use super::GameServer;

impl GameServer {
    /// Set your status
    pub(super) async fn handle_send_ustat(
        &mut self,
        who: usize,
        cid: CID,
        uid: UID,
        stat: Stat,
    ) -> Result<()> {
        // Only allow this if it comes from the same user
        if self.conns[who].cid == cid && self.conns[who].uid == uid {
            let old_stat = self.conns[who].stat;
            self.conns[who].stat = stat;
            debug!("{} stat:{:X} -> {:X}", self.conns[who].name, old_stat, stat);

            // Notify everyone who might care
            let my_mode = self.conns[who].mode;
            let my_lobby = self.conns[who].cur_lobby;

            for conn in &self.conns {
                if conn.cid != cid {
                    if my_mode == conn.mode && my_lobby >= 0 && my_lobby == conn.cur_lobby {
                        conn.write(Packet::SEND_USTAT { cid, uid, stat }).await?;
                    }
                }
            }
        } else {
            warn!(
                "{} tried to change someone else's ustat!",
                self.conns[who].cid
            );
        }
        Ok(())
    }

    /// Allow a player to set their name on their first time playing
    pub(super) async fn handle_set_player_name(
        &mut self,
        who: usize,
        data: SetPlayerName,
    ) -> Result<()> {
        let name = data.name.to_string();
        let name = name.trim().to_string();

        match self
            .db
            .set_player_name(self.conns[who].uid, name.clone())
            .await
        {
            Ok(()) => {
                self.conns[who].name = name;
                self.conns[who]
                    .write(Packet::ACK_SET_CHARACTER_NAME(Status::OK))
                    .await?;
            }
            Err(e) => {
                error!("failed to set player name to {:?}: {:?}", data.name, e);
                self.conns[who]
                    .write(Packet::ACK_SET_CHARACTER_NAME(Status::Err))
                    .await?;
            }
        }

        Ok(())
    }

    /// Fetch user data
    pub(super) async fn handle_req_udata(&self, pid: i16, who: usize, uid: UID) -> Result<()> {
        // TODO: does this work with users who are offline?
        // I think it does

        for conn in &self.conns {
            if conn.uid == uid {
                let packet = Packet::PKT_181(conn.make_udata());
                self.conns[who].write_with_pid(packet, pid).await?;
                return Ok(());
            }
        }

        error!("failed to fetch UDATA for uid={uid}");
        Ok(())
    }

    /// Get the amount of money you have
    pub(super) async fn handle_get_money(&self, pid: i16, who: usize) -> Result<()> {
        self.conns[who]
            .write_with_pid(
                Packet::REP_MONEY {
                    gp: self.conns[who].user.gp,
                    sc: self.conns[who].user.sc,
                },
                pid,
            )
            .await
    }

    /// Get your inventory contents
    pub(super) async fn handle_get_inventory(&self, who: usize) -> Result<()> {
        let packet = Packet::PKT_132 {
            count: self.conns[who].user.inventory.len() as i32,
            items: self.conns[who].user.inventory.clone(),
        };
        self.conns[who].write(packet).await
    }

    /// Get your golfbag contents
    pub(super) async fn handle_get_golfbag(&self, who: usize) -> Result<()> {
        let packet = Packet::PKT_134 {
            x4: 0,
            cid: self.conns[who].cid,
            items: self.conns[who].user.golfbag,
            unk: [0; 4060],
        };
        self.conns[who].write(packet).await
    }

    /// Set your holdbox contents
    pub(super) async fn handle_chg_holdbox(
        &mut self,
        who: usize,
        hold_item: [Item; 8],
    ) -> Result<()> {
        self.conns[who].user.holdbox = hold_item;
        self.save_user(who).await;
        self.conns[who]
            .write(Packet::ACK_CHG_HOLDBOX(Status::OK))
            .await
    }
}
