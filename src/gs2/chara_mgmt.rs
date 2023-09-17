use anyhow::{bail, Result};
use log::{error, warn};

use crate::{
    data::Appearance,
    packets::{ChrUID, Packet, Status, CID},
};

use super::GameServer;

impl GameServer {
    /// Allow a player to create their first character
    pub(super) async fn handle_create_first_character(
        &mut self,
        who: usize,
        appear: Appearance,
    ) -> Result<()> {
        if self.conns[who].user.default_chr_uid != -1 {
            // Fail right away if they already have a character assigned.
            return Ok(());
        }

        match self.db.create_character(self.conns[who].uid, appear).await {
            Ok((chr_uid, character)) => {
                let conn = &mut self.conns[who];

                conn.user.default_chr_uid = chr_uid;
                conn.characters.push((chr_uid, character));

                self.save_user(who).await;
                self.conns[who]
                    .write(Packet::ACK_FIRST_CHARACTER_APPEARANCE(Status::OK))
                    .await?;
            }
            Err(e) => {
                error!("failed to create initial character: {e:?}");
                self.conns[who]
                    .write(Packet::ACK_FIRST_CHARACTER_APPEARANCE(Status::Err))
                    .await?;
            }
        }

        Ok(())
    }

    /// Fetch one character
    pub(super) async fn handle_req_chrdata(
        &self,
        pid: i16,
        who: usize,
        cid: CID,
        chr_uid: ChrUID,
    ) -> Result<()> {
        if let Some(&victim) = self.conn_lookup.get(&cid) {
            let uid = self.conns[victim].uid;
            let mut found = false;

            for (this_chr_uid, chara) in &self.conns[victim].characters {
                if *this_chr_uid == chr_uid {
                    let data = chara.to_chr_data(chr_uid);
                    let packet = Packet::SEND_CHRDATA { cid, uid, data };
                    self.conns[who].write_with_pid(packet, pid).await?;
                    found = true;
                    break;
                }
            }

            if !found {
                warn!("REQ_CHRDATA for known cid {cid} but unknown chr_uid {chr_uid}");
            }
        } else {
            warn!("REQ_CHRDATA for unknown cid {cid}, chr_uid {chr_uid}");
        }

        Ok(())
    }

    /// Fetch all characters belonging to a particular connection
    pub(super) async fn handle_get_chrdata(&self, who: usize, cid: CID) -> Result<()> {
        if let Some(&victim) = self.conn_lookup.get(&cid) {
            let uid = self.conns[victim].uid;
            for (chr_uid, chara) in &self.conns[victim].characters {
                let data = chara.to_chr_data(*chr_uid);
                self.conns[who]
                    .write(Packet::SEND_CHRDATA { cid, uid, data })
                    .await?;
            }
        } else {
            warn!("GET_CHRDATA for unknown cid {cid}");
        }

        Ok(())
    }

    /// Get the appearance for the active character
    pub(super) async fn get_active_appearance(&self, pid: i16, who: usize, cid: CID) -> Result<()> {
        // Find the targeted player
        if let Some(&victim_ind) = self.conn_lookup.get(&cid) {
            let victim = &self.conns[victim_ind];
            for (chr_uid, chara) in &victim.characters {
                if *chr_uid == victim.user.default_chr_uid {
                    let packet = Packet::SEND_APPEAR(cid, 0, chara.appearance.clone());
                    self.conns[who].write_with_pid(packet, pid).await?;
                    break;
                }
            }
        } else {
            warn!("Getting appearance for unknown cid {cid}");
        }

        Ok(())
    }

    /// Get the active character ID for a particular player
    pub(super) async fn handle_get_curr_chr_uid(
        &self,
        pid: i16,
        who: usize,
        cid: CID,
    ) -> Result<()> {
        // Find the targeted player
        if let Some(&victim_ind) = self.conn_lookup.get(&cid) {
            let victim = &self.conns[victim_ind];
            let packet = Packet::SEND_CRCHRUID {
                cid,
                now_chr_uid: victim.user.default_chr_uid,
            };
            self.conns[who].write_with_pid(packet, pid).await?;
        } else {
            warn!("Getting current chr_uid for unknown cid {cid}");
        }

        Ok(())
    }

    /// Write a modified character appearance
    pub(super) async fn handle_req_chg_appear(
        &mut self,
        who: usize,
        cid: CID,
        chr_uid: ChrUID,
        appear: Appearance,
    ) -> Result<()> {
        // Ensure they're only modifying themselves and their own character
        if cid != self.conns[who].cid {
            error!("REQ_CHG_APPEAR for other cid {cid}, chr_uid {chr_uid}");
            self.conns[who].write(Packet::PKT_104(Status::Err)).await?;
        } else {
            let mut found = false;

            for (check_chr_uid, chara) in &mut self.conns[who].characters {
                if *check_chr_uid == chr_uid {
                    // This is the one
                    chara.appearance = appear;
                    self.db.write_character(chr_uid, chara.clone()).await;
                    found = true;
                    break;
                }
            }

            let status = if found { Status::OK } else { Status::Err };
            self.conns[who].write(Packet::PKT_104(status)).await?;
        }

        Ok(())
    }

    /// Write a modified set of character parameters (including equipped ball/club)
    pub(super) async fn handle_req_chg_chr_param(
        &mut self,
        who: usize,
        packet: Packet,
    ) -> Result<()> {
        if let Packet::REQ_CHG_CHR_PARAM {
            chr_uid,
            cr_class,
            power,
            impact,
            params,
            club,
            ball,
            caddie,
        } = packet
        {
            let mut found = false;

            for (check_chr_uid, chara) in &mut self.conns[who].characters {
                if *check_chr_uid == chr_uid {
                    // This is the one
                    chara.class_cap = cr_class;
                    chara.settings = params;
                    chara.club = club;
                    chara.ball = ball;
                    chara.caddie = caddie;

                    self.db.write_character(chr_uid, chara.clone()).await;
                    found = true;
                    break;
                }
            }

            let status = if found { Status::OK } else { Status::Err };
            self.conns[who]
                .write(Packet::ACK_CHG_CHR_PARAM(status))
                .await?;
        } else {
            bail!("bad packet")
        }

        Ok(())
    }
}
