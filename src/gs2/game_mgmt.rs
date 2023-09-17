use anyhow::{bail, Result};
use log::{error, info, warn};
use rand::prelude::*;

use crate::data::{Item, ItemCategory};
use crate::{
    data::CountedItem,
    packets::{Mode, Packet, Status, CID},
};

use super::{lobby_mgmt::Room, GameServer};

fn generate_single_mode_game(cid: CID) -> Packet {
    let mut rng = thread_rng();
    let mut hole_no = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17];
    let wind_dir = [0; 18];
    let wind_pow = [0; 18];
    let weather = [0; 18];
    let cup_pos = [0; 18];
    hole_no.shuffle(&mut rng);
    hole_no[3..].fill(-1);

    let mut cid_array = [-1; 50];
    cid_array[0] = cid;

    Packet::ORD_GAMESTART {
        mode: Mode::Single,
        rule: 0, // strokes
        time: 0, // unlimited
        member: 1,
        member_max: 1,
        course: 0, // southern country
        season: 1, // daytime
        holes: 3,
        hole_no,
        wind_dir,
        wind_pow,
        weather,
        cup_pos,
        cid: cid_array,
        caddies: [0; 50],
        caddie_reliance: [0; 50],
        ball_array: [0; 50],
        hold_box: [[CountedItem::default(); 8]; 50],
    }
}

fn generate_vs_game(room: &Room) -> Packet {
    // TODO: actually use all the interesting parameters in the room config
    // TODO: prefill caddies, ball_array, hold_box with appropriate info from the participants
    let mut rng = thread_rng();
    let mut hole_no = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17];
    let wind_dir = [0; 18];
    let wind_pow = [0; 18];
    let weather = [0; 18];
    let cup_pos = [0; 18];
    hole_no.shuffle(&mut rng);
    hole_no[3..].fill(-1);

    let mut cid_array = [-1; 50];
    for (index, cid) in room.members.iter().enumerate() {
        cid_array[index] = *cid;
    }

    Packet::ORD_GAMESTART {
        mode: Mode::VS,
        rule: 0, // strokes
        time: 0, // unlimited
        member: room.members.len().try_into().unwrap(),
        member_max: room.max_members.try_into().unwrap(),
        course: 0, // southern country
        season: 1, // daytime
        holes: 3,
        hole_no,
        wind_dir,
        wind_pow,
        weather,
        cup_pos,
        cid: cid_array,
        caddies: [0; 50],
        caddie_reliance: [0; 50],
        ball_array: [0; 50],
        hold_box: [[CountedItem::default(); 8]; 50],
    }
}

impl GameServer {
    /// Return the list of Carry Items available in single mode
    pub(super) async fn handle_init_single_mode(&self, who: usize) -> Result<()> {
        let env = ItemCategory::CarryItemEnvironment;
        let gauge = ItemCategory::CarryItemPowerGauge;

        self.conns[who]
            .write(Packet::PKT_275 {
                count: 8,
                items: [
                    CountedItem::new(Item::new(env, 1), 100),
                    CountedItem::new(Item::new(env, 3), 100),
                    CountedItem::new(Item::new(env, 5), 100),
                    CountedItem::new(Item::new(env, 7), 100),
                    CountedItem::new(Item::new(gauge, 1), 100),
                    CountedItem::new(Item::new(gauge, 2), 100),
                    CountedItem::new(Item::new(gauge, 3), 100),
                    CountedItem::new(Item::new(gauge, 4), 100),
                ],
            })
            .await
    }

    pub(super) async fn handle_start_game(&mut self, who: usize) -> Result<()> {
        let mode = self.conns[who].mode;
        let lobby_num = self.conns[who].cur_lobby;
        let room_num = self.conns[who].cur_room;

        match mode {
            Mode::Single => {
                // this is the most basic case
                let packet = generate_single_mode_game(self.conns[who].cid);
                self.conns[who].write(packet).await?;
                self.conns[who]
                    .write(Packet::ACK_GAMESTART(Status::OK))
                    .await?;
            }
            Mode::VS => {
                if let Some(room) = self.lobbies.room_mut(mode, lobby_num, room_num) {
                    let packet = generate_vs_game(room);

                    // Tell every player in the room
                    for cid in &room.members {
                        let victim = self.conn_lookup[cid];
                        self.conns[victim].write(packet.clone()).await?;
                    }

                    // TODO: send EnableCaddieList here based off logs
                    self.conns[who]
                        .write(Packet::ACK_GAMESTART(Status::OK))
                        .await?;
                } else {
                    self.conns[who]
                        .write(Packet::ACK_GAMESTART(Status::Err))
                        .await?;
                    bail!("not in a room");
                }
            }
            _ => {
                bail!("unknown game mode: {mode:?}")
            }
        }

        Ok(())
    }

    /// Sync the "loaded yes/no" flag to the other players in a room
    pub(super) async fn handle_send_loadstat(&self, who: usize, progress: i8) -> Result<()> {
        let my_cid = self.conns[who].cid;
        let packet = Packet::SEND_LOADSTAT(self.conns[who].cid, progress);

        // Relay this to everybody else in the room
        if let Some(room) = self.lobbies.room(
            self.conns[who].mode,
            self.conns[who].cur_lobby,
            self.conns[who].cur_room,
        ) {
            for &cid in &room.members {
                if cid != my_cid {
                    let victim = *self.conn_lookup.get(&cid).unwrap();
                    self.conns[victim].write(packet.clone()).await?;
                }
            }
        } else {
            warn!("received LoadStat for someone who isn't in a room");
        }

        Ok(())
    }

    /// Sync detailed info about loading progress to the other players in a room
    pub(super) async fn handle_send_loadstat2(&self, who: usize, progress: i8) -> Result<()> {
        let my_cid = self.conns[who].cid;
        let packet = Packet::SEND_LOADSTAT2(self.conns[who].cid, progress);

        // Relay this to everybody else in the room
        if let Some(room) = self.lobbies.room(
            self.conns[who].mode,
            self.conns[who].cur_lobby,
            self.conns[who].cur_room,
        ) {
            for &cid in &room.members {
                if cid != my_cid {
                    let victim = *self.conn_lookup.get(&cid).unwrap();
                    self.conns[victim].write(packet.clone()).await?;
                }
            }
        } else {
            warn!("received LoadStat2 for someone who isn't in a room");
        }

        Ok(())
    }

    /// Sync the selected club to the other players in a room
    pub(super) async fn handle_shot_club(&self, who: usize, club: i8) -> Result<()> {
        let packet = Packet::SEND_CRCLUB {
            cid: self.conns[who].cid,
            club,
        };
        self.send_packet_to_roommates(who, packet).await
    }

    /// Sync the shot direction to the other players in a room
    pub(super) async fn handle_shot_dir(&self, who: usize, dir: f32) -> Result<()> {
        let packet = Packet::SEND_DIRECTION {
            cid: self.conns[who].cid,
            dir,
        };
        self.send_packet_to_roommates(who, packet).await
    }

    /// Sync the shot info to the other players in a room
    pub(super) async fn handle_shot_info(
        &mut self,
        who: usize,
        clock: u64,
        dir: f32,
        power: i16,
        impact: i16,
        hit_x: i8,
        hit_y: i8,
        club: i8,
    ) -> Result<()> {
        // Keep track of who made this shot
        if let Some(room) = self.lobbies.room_mut(
            self.conns[who].mode,
            self.conns[who].cur_lobby,
            self.conns[who].cur_room,
        ) {
            room.current_player = self.conns[who].cid;
        }

        let packet = Packet::SEND_SHOT {
            clock,
            cid: self.conns[who].cid,
            dir,
            power,
            impact,
            hit_x,
            hit_y,
            club,
        };
        self.send_packet_to_roommates(who, packet).await
    }

    /// Sync the ball position to the other players in a room
    pub(super) async fn handle_ballpos(
        &self,
        who: usize,
        hole: i8,
        stat: i8,
        x: f32,
        y: f32,
        z: f32,
    ) -> Result<()> {
        let packet = Packet::SEND_BALLPOS {
            cid: self.conns[who].cid,
            hole,
            stat,
            x,
            y,
            z,
        };
        self.send_packet_to_roommates(who, packet).await
    }

    /// Sync the ball stop position to the players in a room
    pub(super) async fn handle_stop_ballpos(
        &self,
        who: usize,
        hole: i8,
        stat: i8,
        x: f32,
        y: f32,
        z: f32,
    ) -> Result<()> {
        if let Some(room) = self.lobbies.room(
            self.conns[who].mode,
            self.conns[who].cur_lobby,
            self.conns[who].cur_room,
        ) {
            let my_cid = self.conns[who].cid;
            let player_cid = room.current_player;
            if my_cid != player_cid {
                error!("player {my_cid} tried to send STOP_BALLPOS but they're not {player_cid}!");
                return Ok(());
            }
        }

        let packet = Packet::SEND_STOP_BALLPOS {
            cid: self.conns[who].cid,
            hole,
            stat,
            x,
            y,
            z,
        };
        // The client expects to *receive* it too, it seems
        self.conns[who].write(packet.clone()).await?;
        self.send_packet_to_roommates(who, packet).await
    }

    /// Send a command to the players in a room
    pub(super) async fn handle_send_pcommand(
        &self,
        who: usize,
        cmd_and_flag: u16,
        p0: u32,
        p1: u32,
    ) -> Result<()> {
        let packet = Packet::SEND_PCOMMAND {
            cid: self.conns[who].cid,
            p0,
            p1,
            cmd_and_flag,
        };

        // no fucking clue if this cmd flag thing is correct lmao
        if (cmd_and_flag & 0x8000) == 0 {
            // The client expects to *receive* it too, it seems
            self.conns[who].write(packet.clone()).await?;
        }

        self.send_packet_to_roommates(who, packet).await
    }

    async fn send_packet_to_roommates(&self, who: usize, packet: Packet) -> Result<()> {
        let my_cid = self.conns[who].cid;

        if let Some(room) = self.lobbies.room(
            self.conns[who].mode,
            self.conns[who].cur_lobby,
            self.conns[who].cur_room,
        ) {
            for &cid in &room.members {
                if cid != my_cid {
                    let victim = *self.conn_lookup.get(&cid).unwrap();
                    self.conns[victim].write(packet.clone()).await?;
                }
            }

            Ok(())
        } else {
            bail!("user is not in a room!")
        }
    }
}
