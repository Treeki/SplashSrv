use std::collections::BTreeMap;
use std::sync::Arc;

use anyhow::Result;
use log::{error, info, warn};
use tokio::net::{TcpListener, ToSocketAddrs};
use tokio::sync::{mpsc, oneshot};
use tokio_rustls::rustls::ServerConfig;
use tokio_rustls::TlsAcceptor;

use crate::data::shop::{build_salon_list, build_sell_list};
use crate::data::{Character, SellItem, User};
use crate::db_task::DBTask;
use crate::packets::{
    AckIDPassResult, ChrUID, Element, IDPass, LobbyNum, Mode, ModeCtrl, Packet, RoomNum, Stat,
    UData, UList, UListL, CID, UID,
};

use self::conn_task::{ConnReceiver, ConnSender};

mod chara_mgmt;
mod conn_task;
mod game_mgmt;
mod lobby_mgmt;
mod record_mgmt;
mod shop_mgmt;
mod user_mgmt;

enum Message {
    Login(IDPass, oneshot::Sender<LoginResult>),
    PlayerData { cid: CID, pid: i16, packet: Packet },
    Logout(CID),
}

#[derive(Debug)]
enum LoginResult {
    Success { cid: CID, packet_rx: ConnReceiver },
    Fail(AckIDPassResult),
}

/// Data for a player who has successfully authenticated to the game server
struct Player {
    cid: CID,
    uid: UID,
    name: String,
    user: User,
    characters: Vec<(ChrUID, Character)>,
    cur_lobby: LobbyNum,
    cur_room: RoomNum,
    stat: Stat,
    mode: Mode,
    packet_tx: ConnSender,
}

impl Player {
    fn make_udata(&self) -> UData {
        UData {
            cid: self.cid,
            uid: self.uid,
            chr_uid: self.user.default_chr_uid,
            golfbag: self.user.golfbag,
            holdbox: self.user.holdbox,
            medals: [[0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]],
            // these are all the amounts of awards received for tournaments
            awards: [0; 20],
            rank_score_item_on: 0,
            rank_score_item_off: 0,
            mp: 0,
            year: 2023,
            month: 8,
            day: 23,
            name: self.name.parse().unwrap(),
            element: self.user.element,
            class: self.user.class,
            // *** check GetPlayerGrade func for these ***
            rank_item_on: 0,  // PlayerGrade
            rank_item_off: 0, // PlayerGrade
            best_rank_item_on: 0,
            best_rank_item_off: 0,
            x_f4: 0,
            debug: false,
        }
    }

    fn make_ulist(&self) -> UList {
        UList {
            cid: self.cid,
            uid: self.uid,
            stat: self.stat.bits() as u16,
            team: 0, // fix me
            mode: self.mode,
            lobby: self.cur_lobby,
            room: self.cur_room,
            pclass: self.user.class.class(), // TODO is this the selected class?
            element: self.user.element,
            title: 0, // todo
            sv_no: 0,
            circle: 0,
            name: self.name.parse().unwrap(),
        }
    }

    fn make_ulist_l(&self) -> UListL {
        UListL {
            cid: self.cid,
            uid: self.uid,
            stat: self.stat.bits() as u16,
            team: 0, // fix me
            mode: self.mode,
            lobby: self.cur_lobby,
            room: self.cur_room,
            pclass: self.user.class.class(), // TODO is this the selected class?
            element: self.user.element,
            title: 0, // todo
            circle: 0,
            name: self.name.parse().unwrap(),
        }
    }

    async fn write(&self, packet: Packet) -> Result<()> {
        Ok(self.packet_tx.send((None, packet)).await?)
    }

    async fn write_with_pid(&self, packet: Packet, pid: i16) -> Result<()> {
        Ok(self.packet_tx.send((Some(pid), packet)).await?)
    }
}

struct GameServer {
    next_cid: CID,
    conns: Vec<Player>,
    conn_lookup: BTreeMap<CID, usize>,
    lobbies: lobby_mgmt::Lobbies,
    shop_items: Vec<SellItem>,
    salon_items: Vec<SellItem>,
    db: DBTask,
}

impl GameServer {
    fn generate_cid(&mut self) -> CID {
        loop {
            let cid = self.next_cid;
            self.next_cid += 1;
            if self.next_cid > 999 {
                self.next_cid = 600;
            }

            if !self.conn_lookup.contains_key(&cid) {
                return cid;
            }
        }
    }

    /// Asynchronously write a user's data back to the database.
    async fn save_user(&self, who: usize) {
        let conn = &self.conns[who];
        self.db.write_user(conn.uid, conn.user.clone()).await;
    }

    /// Try and switch a player to a different game mode.
    async fn handle_change_mode(&mut self, who: usize, new_mode: Mode) -> Result<()> {
        let cid = self.conns[who].cid;
        let old_mode = self.conns[who].mode;
        info!("📦 {cid} changing from mode {old_mode:?} to {new_mode:?}");

        if old_mode != new_mode {
            if self.conns[who].cur_lobby >= 0 {
                self.eject_from_lobby(who).await?;
            }

            self.conns[who].mode = new_mode;
        }

        self.conns[who]
            .write(Packet::ACK_CHG_MODE(new_mode))
            .await?;
        Ok(())
    }

    /// Try and add a player to the server.
    async fn handle_login(&mut self, p: IDPass) -> LoginResult {
        let login_id = p.username.to_string();
        let password = p.password.to_string();

        let account = match self.db.authenticate_user_to_game(login_id, password).await {
            Ok(account) => account,
            Err(e) => {
                error!("failed to auth {p:?}: {e:?}");
                return LoginResult::Fail(AckIDPassResult::IDError);
            }
        };

        // Is this user already logged in?
        for conn in &self.conns {
            if conn.uid == account.uid {
                return LoginResult::Fail(AckIDPassResult::MultiLoginError);
            }
        }

        // All checks out
        let cid = self.generate_cid();
        let (packet_tx, packet_rx) = mpsc::channel(128);
        let name = account
            .name
            .unwrap_or_else(|| format!("_{}", p.username.to_string()));

        let who = self.conns.len();
        let player = Player {
            cid,
            uid: account.uid,
            name,
            user: account.user,
            characters: account.characters,
            cur_lobby: -1,
            cur_room: -1,
            stat: Stat::empty(),
            mode: Mode::None,
            packet_tx,
        };

        // Send their initial packets
        player
            .write(Packet::ACK_IDPASS_G(player.make_udata()))
            .await
            .unwrap();
        player
            .write(Packet::ORD_COLOR_RESULT {
                element: Element::None,
                last_element: Element::None,
                color_result: 0,
                rank_in_color: 0,
                gp: 0,
                item: Default::default(),
            })
            .await
            .unwrap();

        self.conns.push(player);
        self.conn_lookup.insert(cid, who);

        LoginResult::Success { cid, packet_rx }
    }

    /// Remove a player from the server and disconnect them.
    async fn remove_player(&mut self, cid: CID) -> Result<()> {
        match self.conn_lookup.remove(&cid) {
            Some(who) => {
                info!("removing player cid:{cid}");

                if self.conns[who].cur_lobby >= 0 {
                    self.eject_from_lobby(who).await?;
                }

                let player = self.conns.swap_remove(who);

                // swap_remove may have moved a player from the end to 'who'.
                // If this occurs, we need to fix their entry in the lookup map.
                if let Some(pawn) = self.conns.get(who) {
                    let old_entry = self.conn_lookup.insert(pawn.cid, who);
                    assert_eq!(old_entry, Some(self.conns.len()));
                }

                // So long, gay Bowser
                // Their connection will be dropped once player is dropped
                info!("goodbye, {}", player.name);
            }
            None => {
                error!("⚠️ logout for unknown user {cid}");
            }
        }

        Ok(())
    }

    /// Handle a packet received from a player, dispatching to other components as necessary.
    async fn handle_player_data(&mut self, who: usize, pid: i16, packet: Packet) -> Result<()> {
        use Packet::*;

        info!("[{}] {:?}", self.conns[who].cid, packet);

        match packet {
            REQ_CHG_MODE(mode) => self.handle_change_mode(who, mode).await?,
            GET_LOBBY_NUM => self.handle_get_lobby_num(who).await?,
            GET_LOBBY_DATA { index, mode } => {
                self.handle_get_lobby_data(pid, who, index, mode).await?
            }
            REQ_ENTER_LOBBY(index) => self.handle_enter_lobby(who, index).await?,
            REQ_MAKE_ROOM(data) => self.handle_make_room(pid, who, data).await?,
            GET_ROOMS => self.handle_get_rooms(pid, who).await?,
            REQ_ENTER_ROOM {
                room,
                unk_room_flag: _,
                room_password,
            } => {
                self.handle_enter_room(pid, who, room, &room_password.to_string())
                    .await?
            }
            REQ_ULIST(mode, lobby, room) => {
                self.handle_get_room_members(pid, who, mode, lobby, room)
                    .await?
            }
            // 24 - exit room
            SEND_USTAT { cid, uid, stat } => self.handle_send_ustat(who, cid, uid, stat).await?,
            // 27 - SEND_MESSAGE
            // 28 - update room
            REQ_GAMESTART => self.handle_start_game(who).await?,
            CLIENT_CRCLUB(club) => self.handle_shot_club(who, club).await?,
            CLIENT_DIRECTION(dir) => self.handle_shot_dir(who, dir).await?,
            CLIENT_SHOT {
                clock,
                server_cid: _,
                dir,
                power,
                impact,
                hit_x,
                hit_y,
                club,
            } => {
                self.handle_shot_info(who, clock, dir, power, impact, hit_x, hit_y, club)
                    .await?
            }
            // 39 - send score
            // 40 - request URecord
            REQ_CRECORD {
                uid,
                course,
                season,
                hole_idx,
            } => {
                self.handle_get_c_record(pid, who, uid, course, season, hole_idx)
                    .await?
            }
            CLIENT_LOADSTAT(progress) => self.handle_send_loadstat(who, progress).await?,
            CLIENT_BALLPOS {
                server_cid: _,
                hole,
                stat,
                x,
                y,
                z,
            } => self.handle_ballpos(who, hole, stat, x, y, z).await?,
            // 48 - holeout
            // 50 - ready for quick matching
            // 52 - un-ready for quick matching
            // 55 - rank jump complete
            // 56 - start quick matching game
            // 65 - look up player by UID?
            // 67 - look up player by name?
            // 69 - send friend req
            // 71 - get friends
            // 73 - get inbound requests
            // 75 - get outbound requests
            // 77 - accept/deny request
            // 79 - remove friend
            // 81 - cancel request
            REQ_APPEAR(cid) => self.get_active_appearance(pid, who, cid).await?,
            // 85 - chrpos
            REQ_ULIST_L(mode, index) => {
                self.handle_req_lobby_members(pid, who, index, mode).await?
            }
            PKT_89 => self.handle_get_sell_items(who).await?,
            REQ_BUY_ITEM(item) => self.handle_buy_item(who, item).await?,
            PKT_93 => self.handle_get_money(pid, who).await?,
            SET_FIRST_CHARACTER_APPEARANCE(appear) => {
                self.handle_create_first_character(who, appear).await?
            }
            // 97 - ? uid ?
            // 98 - ? cid ?
            REQ_CHRDATA { cid, chr_uid } => self.handle_req_chrdata(pid, who, cid, chr_uid).await?,
            GET_CHRDATA(cid) => self.handle_get_chrdata(who, cid).await?,
            REQ_CHG_APPEAR {
                cid,
                chr_uid,
                appear,
            } => {
                self.handle_req_chg_appear(who, cid, chr_uid, appear)
                    .await?
            }
            SET_PLAYER_NAME(data) => self.handle_set_player_name(who, data).await?,

            // 107 - gets global course record
            // 109 - REQ_UNRECEIVE_SMAIL_CNT
            // 111 - also mail related
            // 113 - gets a mail
            // 117 - REQ_BLOCKLIST
            // 119 - block user
            // 121 - unblock user
            // 123 - search players
            // 125 - some stat update

            // 126 - CLIENT_CUP_IN
            // 128 - REP_CLOCK
            // 129 - room search
            PKT_131(_) => self.handle_get_inventory(who).await?,
            PKT_133(_) => self.handle_get_golfbag(who).await?,

            CLIENT_PCOMMAND {
                server_cid: _,
                p0,
                p1,
                cmd_and_flag,
            } => self.handle_send_pcommand(who, cmd_and_flag, p0, p1).await?,
            PKT_137(cid) => self.handle_get_curr_chr_uid(pid, who, cid).await?,

            // 138 - REQ_CHG_CRCHRUID
            // 141 - ?
            REQ_CHG_CHR_PARAM { .. } => self.handle_req_chg_chr_param(who, packet).await?,
            // 147 - get sell caddies
            // 149 - delivery related
            // 151 - employ a caddy
            // 153 - get caddie data?
            // 155 - use item?
            // 158 - send delivery
            // 160 - another delivery thing
            // 162 - get macro data
            // 164 - store macro
            PKT_166 => self.handle_get_salon_items(who).await?,
            // 168 - buy salon item
            // 170 - get title list
            // 172 - get title
            // 174 - REQ_CHG_TITLE
            // 176 - client-side send telop
            // 179 - CompeLounge related
            REQ_UDATA(uid) => self.handle_req_udata(pid, who, uid).await?,
            // 182 - request ranking
            CLIENT_LOADSTAT2(progress) => self.handle_send_loadstat2(who, progress).await?,

            PKT_189 { hold_item } => self.handle_chg_holdbox(who, hold_item).await?,

            // 192 - game centre/code centre related
            // 194 - send command 2
            // 196 - buy item by ticket
            // 198 - play UFO game
            // 200 - employ caddy by ticket
            // 202 - buy salon item by ticket
            // 204 - get NP?
            // 208 - buy item by NP
            // 211 - set team
            // 213 - play slots game
            // 215 - set quick settings itemon
            // 216 - REQ_CHG_OWNER?
            // 217 - accept/deny owner transfer?
            // 219 - kick user?
            // 222 - ReqChgCaddieByItem
            // 227 - GameCenter get number of plays?
            // 229 - one type of ping
            // 232 - update game options
            CLIENT_STOP_BALLPOS {
                server_cid: _,
                hole,
                stat,
                x,
                y,
                z,
            } => self.handle_stop_ballpos(who, hole, stat, x, y, z).await?,

            // 238 - REQ_ADD_GP
            // 240 - reload room data?
            // 241 - CaddieItemRecoveryOB_Task ItemUseRequest - USE_HOLDITEM?
            // 246 - return lounge all
            // 250 - REQ_PING
            // 263 - init recycle system
            // 266 - start recycling
            GET_MODECTRL => {
                let modectrl = ModeCtrl { flags: [true; 92] };
                self.conns[who]
                    .write(Packet::SEND_MODECTRL(modectrl))
                    .await?;
            }

            // 270 - redeem code
            // 272 - redeem code
            PKT_274 => self.handle_init_single_mode(who).await?,

            // 276 - trash items
            // 279 - send invite
            // 283 - GG CSAuth response
            // 286 - retire?
            // 308 - REQ_SVITEMDATA
            // 311 - REQ_CLUBDATA
            // 316 - debug message
            _ => {
                error!("🔥 unhandled!");
            }
        }

        Ok(())
    }

    fn start(db: DBTask) -> mpsc::Sender<Message> {
        let (msg_tx, mut msg_rx) = mpsc::channel(1024);

        tokio::spawn(async move {
            let mut gs = GameServer {
                next_cid: 600,
                conns: Vec::new(),
                conn_lookup: BTreeMap::new(),
                lobbies: lobby_mgmt::create_initial_lobbies(),
                shop_items: build_sell_list(),
                salon_items: build_salon_list(),
                db,
            };

            while let Some(msg) = msg_rx.recv().await {
                match msg {
                    Message::Login(p, resp) => {
                        let result = gs.handle_login(p).await;
                        // something has gone really wrong if the receiver has been dropped
                        resp.send(result)
                            .expect("LoginResult should always be received!");
                    }

                    Message::Logout(cid) => {
                        if let Err(e) = gs.remove_player(cid).await {
                            error!("🔥🔥🔥🔥🔥 failed while removing player {cid} 🔥🔥🔥🔥🔥");
                            error!("{e:?}");
                        }
                    }

                    Message::PlayerData { cid, pid, packet } => match gs.conn_lookup.get(&cid) {
                        Some(&who) => {
                            if let Err(e) = gs.handle_player_data(who, pid, packet).await {
                                error!("error while handling pid={pid} from cid={cid}: {e:?}");
                            }
                        }
                        None => {
                            warn!("👻 received spooky packet from unknown player with cid={cid}");
                        }
                    },
                }
            }
        });

        msg_tx
    }
}

pub async fn run<A: ToSocketAddrs>(db: DBTask, config: Arc<ServerConfig>, addr: A) -> Result<()> {
    let acceptor = TlsAcceptor::from(config);
    let listener = TcpListener::bind(addr).await?;

    let gs2 = GameServer::start(db);

    loop {
        let (stream, _) = listener.accept().await?;
        let acceptor = acceptor.clone();
        let gs2 = gs2.clone();

        conn_task::run_connection(gs2, stream, acceptor);
    }
}
