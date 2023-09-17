use anyhow::{bail, Result};
use log::error;
use thiserror::Error;

use crate::packets::{LobbyData, LobbyNum, Mode, Packet, Packet19, RoomNum, RoomStat, Status, CID};

use super::GameServer;

#[derive(Error, Debug)]
enum EnterRoomError {
    #[error("player is already in a room")]
    AlreadyInRoom,
    #[error("lobby or room does not exist")]
    RoomNotFound,
    #[error("room is full")]
    RoomIsFull,
    #[error("incorrect password specified")]
    WrongPassword,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub(super) struct Lobbies {
    vs_lobbies: Vec<Lobby>,
    compe_lobbies: Vec<Lobby>,
}

impl Lobbies {
    fn lobbies(&self, mode: Mode) -> Option<&Vec<Lobby>> {
        match mode {
            Mode::VS => Some(&self.vs_lobbies),
            Mode::Competition => Some(&self.compe_lobbies),
            _ => None,
        }
    }

    fn lobbies_mut(&mut self, mode: Mode) -> Option<&mut Vec<Lobby>> {
        match mode {
            Mode::VS => Some(&mut self.vs_lobbies),
            Mode::Competition => Some(&mut self.compe_lobbies),
            _ => None,
        }
    }

    fn lobby(&self, mode: Mode, num: LobbyNum) -> Option<&Lobby> {
        let lobbies = self.lobbies(mode)?;
        if num >= 0 && (num as usize) < lobbies.len() {
            Some(&lobbies[num as usize])
        } else {
            None
        }
    }

    fn lobby_mut(&mut self, mode: Mode, num: LobbyNum) -> Option<&mut Lobby> {
        let lobbies = self.lobbies_mut(mode)?;
        if num >= 0 && (num as usize) < lobbies.len() {
            Some(&mut lobbies[num as usize])
        } else {
            None
        }
    }

    pub(super) fn room(&self, mode: Mode, lobby_num: LobbyNum, room_num: RoomNum) -> Option<&Room> {
        let lobby = self.lobby(mode, lobby_num)?;
        match lobby.rooms.binary_search_by_key(&room_num, |r| r.room_num) {
            Ok(index) => Some(&lobby.rooms[index]),
            Err(_) => None,
        }
    }

    pub(super) fn room_mut(
        &mut self,
        mode: Mode,
        lobby_num: LobbyNum,
        room_num: RoomNum,
    ) -> Option<&mut Room> {
        let lobby = self.lobby_mut(mode, lobby_num)?;
        match lobby.rooms.binary_search_by_key(&room_num, |r| r.room_num) {
            Ok(index) => Some(&mut lobby.rooms[index]),
            Err(_) => None,
        }
    }
}

pub(super) struct Lobby {
    name: String,
    members: Vec<CID>,
    max_members: usize,
    rooms: Vec<Room>,
}

pub(super) struct Room {
    pub(super) room_num: RoomNum,
    pub(super) members: Vec<CID>,
    pub(super) max_members: usize,
    pub(super) name: String,
    pub(super) password: Option<String>,
    pub(super) allow_spectators: bool,
    pub(super) rules: i8,
    pub(super) course: i8,
    pub(super) season: i8,
    pub(super) time_limit: i8,
    pub(super) num_holes: i8,
    pub(super) course_setting: i8,
    pub(super) limit_0: u8,
    pub(super) limit_1: u8,
    pub(super) limit_2: u8,
    pub(super) limit_3: u8,
    pub(super) limit_4: u8,
    pub(super) limit_5: u8,
    pub(super) limit_6: u8,
    pub(super) limit_7: u8,
    pub(super) limit_b_0: u8,
    pub(super) limit_b_1: u8,
    pub(super) limit_b_2: u8,
    pub(super) limit_b_3: u8,
    pub(super) limit_b_4: u8,
    pub(super) current_player: CID,
}

impl Lobby {
    fn make_lobby_data(&self, mode: Mode, num: LobbyNum) -> LobbyData {
        LobbyData {
            num,
            member_max: self.max_members.try_into().unwrap(),
            member: self.members.len().try_into().unwrap(),
            name: self.name.parse().unwrap(),
            unk: [0; 32],
            mode,
        }
    }

    fn pick_free_room_num(&self) -> Option<RoomNum> {
        // assume that our list of rooms is sorted by number!

        let mut candidate = 0;

        for room in &self.rooms {
            if room.room_num > candidate {
                // 'candidate' is definitely free
                break;
            } else if room.room_num == 127 {
                // we're out of space
                return None;
            } else {
                // keep looking
                candidate = room.room_num + 1;
            }
        }

        // if we got here, then 'candidate' should be free
        Some(candidate)
    }
}

impl Room {
    fn new(room_num: RoomNum, data: Packet19) -> Room {
        let password = if (data.room_stat.flag & 4) != 0 {
            Some(data.room_password.to_string())
        } else {
            None
        };

        Room {
            room_num,
            members: Vec::new(),
            max_members: data.room_stat.member_max as usize,
            name: data.room_name.to_string(),
            password,
            allow_spectators: (data.room_stat.flag & 2) != 0,
            rules: data.room_stat.rules,
            course: data.room_stat.course,
            season: data.room_stat.season,
            time_limit: data.room_stat.time_limit,
            num_holes: data.room_stat.num_holes,
            course_setting: data.room_stat.course_setting,
            limit_0: data.room_stat.limit_0,
            limit_1: data.room_stat.limit_1,
            limit_2: data.room_stat.limit_2,
            limit_3: data.room_stat.limit_3,
            limit_4: data.room_stat.limit_4,
            limit_5: data.room_stat.limit_5,
            limit_6: data.room_stat.limit_6,
            limit_7: data.room_stat.limit_7,
            limit_b_0: data.room_stat.limit_b_0,
            limit_b_1: data.room_stat.limit_b_1,
            limit_b_2: data.room_stat.limit_b_2,
            limit_b_3: data.room_stat.limit_b_3,
            limit_b_4: data.room_stat.limit_b_4,
            current_player: -1,
        }
    }

    fn make_room_stat(&self) -> RoomStat {
        // TODO: add flag 1 here, "in round"?
        let flag =
            if self.allow_spectators { 2 } else { 0 } | if self.password.is_some() { 4 } else { 0 };

        RoomStat {
            room: self.room_num,
            flag,
            member_max: self.max_members.try_into().unwrap(),
            member: self.members.len().try_into().unwrap(),
            watcher: 0, // i think this is a count?
            rules: self.rules,
            time_limit: self.time_limit,
            course: self.course,
            season: self.season,
            num_holes: self.num_holes,
            course_setting: self.course_setting,
            limit_0: self.limit_0,
            limit_1: self.limit_1,
            limit_2: self.limit_2,
            limit_3: self.limit_3,
            limit_4: self.limit_4,
            limit_5: self.limit_5,
            limit_6: self.limit_6,
            limit_7: self.limit_7,
            limit_b_0: self.limit_b_0,
            limit_b_1: self.limit_b_1,
            limit_b_2: self.limit_b_2,
            limit_b_3: self.limit_b_3,
            limit_b_4: self.limit_b_4,
        }
    }
}

impl GameServer {
    pub(super) async fn handle_get_lobby_num(&self, who: usize) -> Result<()> {
        if let Some(count) = self.lobbies.lobbies(self.conns[who].mode).map(Vec::len) {
            let packet = Packet::SEND_LOBBY_NUM(count.try_into()?);
            self.conns[who].write(packet).await?;
        }
        Ok(())
    }

    pub(super) async fn handle_get_lobby_data(
        &self,
        pid: i16,
        who: usize,
        num: LobbyNum,
        mode: Mode,
    ) -> Result<()> {
        if let Some(lobby) = self.lobbies.lobby(mode, num) {
            let data = lobby.make_lobby_data(mode, num);
            self.conns[who]
                .write_with_pid(Packet::SEND_LOBBY_DATA(data), pid)
                .await?;
        }

        Ok(())
    }

    pub(super) async fn handle_enter_lobby(&mut self, who: usize, num: LobbyNum) -> Result<()> {
        if self.conns[who].cur_lobby >= 0 {
            bail!("trying to join lobby {num} while already in a lobby!");
        }

        let lobby = match self.lobbies.lobby_mut(self.conns[who].mode, num) {
            Some(lobby) => lobby,
            None => bail!("invalid lobby"),
        };

        // is there space?
        if lobby.members.len() >= lobby.max_members {
            self.conns[who].write(Packet::ACK_ENTER_LOBBY(-1)).await?;
            return Ok(());
        }

        // add this dude
        lobby.members.push(self.conns[who].cid);
        self.conns[who].cur_lobby = num;
        self.conns[who].write(Packet::ACK_ENTER_LOBBY(num)).await?;

        // Notify all other users in the lobby
        let ulist_l = self.conns[who].make_ulist_l();
        let my_cid = self.conns[who].cid;
        for &cid in &lobby.members {
            if cid != my_cid {
                let member_index = *self.conn_lookup.get(&cid).unwrap();
                self.conns[member_index]
                    .write(Packet::SEND_ULIST_L(ulist_l.clone()))
                    .await?;
            }
        }

        Ok(())
    }

    /// Kick a player out of the lobby that they're in
    pub(super) async fn eject_from_lobby(&mut self, who: usize) -> Result<()> {
        let lobby = match self
            .lobbies
            .lobby_mut(self.conns[who].mode, self.conns[who].cur_lobby)
        {
            Some(lobby) => lobby,
            None => bail!("invalid lobby"),
        };

        let cid = self.conns[who].cid;
        let pos = lobby.members.iter().position(|c| *c == cid).unwrap();
        lobby.members.remove(pos);

        self.conns[who].cur_lobby = -1;

        // Notify all other users in the lobby
        let ulist_l = self.conns[who].make_ulist_l();
        for &cid in &lobby.members {
            let member_index = *self.conn_lookup.get(&cid).unwrap();
            self.conns[member_index]
                .write(Packet::SEND_ULIST_L(ulist_l.clone()))
                .await?;
        }

        Ok(())
    }

    /// Get the list of players in the lobby
    pub(super) async fn handle_req_lobby_members(
        &self,
        pid: i16,
        who: usize,
        num: LobbyNum,
        mode: Mode,
    ) -> Result<()> {
        if let Some(lobby) = self.lobbies.lobby(mode, num) {
            for &cid in &lobby.members {
                let member_index = *self.conn_lookup.get(&cid).unwrap();
                let packet = Packet::SEND_ULIST_L(self.conns[member_index].make_ulist_l());
                self.conns[who].write_with_pid(packet, pid).await?;
            }
            Ok(())
        } else {
            bail!("invalid lobby")
        }
    }

    /// Allow players to make rooms
    pub(super) async fn handle_make_room(
        &mut self,
        pid: i16,
        who: usize,
        data: Packet19,
    ) -> Result<()> {
        let lobby = match self.lobbies.lobby_mut(data.mode, data.lobby) {
            Some(lobby) => lobby,
            None => bail!("invalid lobby"),
        };

        if self.conns[who].mode != data.mode {
            bail!("user isn't in the mode")
        }
        if self.conns[who].cur_lobby != data.lobby {
            bail!("user isn't in the lobby")
        }
        if self.conns[who].cur_room >= 0 {
            bail!("user is already in a room")
        }

        // allocate a number for the room
        let room_num = match lobby.pick_free_room_num() {
            Some(n) => n,
            None => {
                error!("failed to create room, all slots full");
                let packet = Packet::ACK_MAKE_ROOM(-1);
                self.conns[who].write_with_pid(packet, pid).await?;
                return Ok(());
            }
        };

        let mut room = Room::new(room_num, data);

        // player will be in the new room by default
        room.members.push(self.conns[who].cid);
        self.conns[who].cur_room = room_num;

        lobby.rooms.push(room);
        lobby.rooms.sort_by_key(|l| l.room_num);

        // inform them of success
        let packet = Packet::ACK_MAKE_ROOM(room_num);
        self.conns[who].write_with_pid(packet, pid).await?;

        Ok(())
    }

    /// Tell a lobby entrant about the rooms that exist
    pub(super) async fn handle_get_rooms(&self, pid: i16, who: usize) -> Result<()> {
        let lobby = match self
            .lobbies
            .lobby(self.conns[who].mode, self.conns[who].cur_lobby)
        {
            Some(lobby) => lobby,
            None => bail!("invalid lobby"),
        };

        for room in &lobby.rooms {
            let data = Packet19 {
                mode: self.conns[who].mode,
                lobby: self.conns[who].cur_lobby,
                room_name: room.name.parse()?,
                room_password: match &room.password {
                    Some(p) => p.parse()?,
                    None => "".parse()?,
                },
                room_stat: room.make_room_stat(),
            };
            self.conns[who]
                .write_with_pid(Packet::PKT_19(data), pid)
                .await?;
        }

        Ok(())
    }

    async fn _enter_room_internal(
        &mut self,
        pid: i16,
        who: usize,
        room_num: RoomNum,
        password: &str,
    ) -> Result<(), EnterRoomError> {
        let mode = self.conns[who].mode;
        let lobby_num = self.conns[who].cur_lobby;

        if self.conns[who].cur_room > 0 {
            return Err(EnterRoomError::AlreadyInRoom);
        }

        let room = self
            .lobbies
            .room_mut(mode, lobby_num, room_num)
            .ok_or(EnterRoomError::RoomNotFound)?;

        if let Some(pw) = &room.password {
            if password != pw {
                return Err(EnterRoomError::WrongPassword);
            }
        }

        if room.members.len() >= room.max_members {
            return Err(EnterRoomError::RoomIsFull);
        }

        // If all that succeeded, we can put them in
        room.members.push(self.conns[who].cid);
        self.conns[who].cur_room = room_num;

        let data = Packet19 {
            mode,
            lobby: lobby_num,
            room_name: room.name.parse().unwrap(),
            room_password: match &room.password {
                Some(p) => p.parse()?,
                None => "".parse()?,
            },
            room_stat: room.make_room_stat(),
        };
        self.conns[who]
            .write_with_pid(Packet::ACK_ENTER_ROOM(data), pid)
            .await?;

        // Notify all other users in the room
        let ulist = self.conns[who].make_ulist();
        let my_cid = self.conns[who].cid;
        for &cid in &room.members {
            if cid != my_cid {
                let member_index = *self.conn_lookup.get(&cid).unwrap();
                self.conns[member_index]
                    .write(Packet::SEND_ULIST(ulist.clone()))
                    .await?;
            }
        }

        Ok(())
    }

    /// Allow players to enter a room
    pub(super) async fn handle_enter_room(
        &mut self,
        pid: i16,
        who: usize,
        room_num: RoomNum,
        password: &str,
    ) -> Result<()> {
        if let Err(e) = self
            ._enter_room_internal(pid, who, room_num, password)
            .await
        {
            error!("failed to enter room: {e:?}");

            let code = if let EnterRoomError::WrongPassword = &e {
                -3
            } else {
                -1
            };
            let data =
                Packet19::create_error(self.conns[who].mode, self.conns[who].cur_lobby, code);
            self.conns[who]
                .write_with_pid(Packet::ACK_ENTER_ROOM(data), pid)
                .await?;
        }

        Ok(())
    }

    /// List the players in a particular room
    pub(super) async fn handle_get_room_members(
        &self,
        pid: i16,
        who: usize,
        mode: Mode,
        lobby_num: LobbyNum,
        room_num: RoomNum,
    ) -> Result<()> {
        if let Some(room) = self.lobbies.room(mode, lobby_num, room_num) {
            for cid in &room.members {
                let member_index = *self.conn_lookup.get(cid).unwrap();
                let packet = Packet::SEND_ULIST(self.conns[member_index].make_ulist());
                self.conns[who].write_with_pid(packet, pid).await?;
            }

            // it seems like we need this to get out of the loading screen
            self.conns[who]
                .write_with_pid(Packet::ACK_ULIST_R(Status::OK), pid)
                .await?;
        } else {
            // room not found
            self.conns[who]
                .write_with_pid(Packet::ACK_ULIST_R(Status::Err), pid)
                .await?;
        }

        Ok(())
    }
}

pub(super) fn create_initial_lobbies() -> Lobbies {
    let vs_lobbies = vec![Lobby {
        name: "Foo".to_string(),
        members: Vec::new(),
        max_members: 10,
        rooms: Vec::new(),
    }];

    let compe_lobbies = vec![Lobby {
        name: "Bar".to_string(),
        members: Vec::new(),
        max_members: 10,
        rooms: Vec::new(),
    }];

    Lobbies {
        vs_lobbies,
        compe_lobbies,
    }
}
