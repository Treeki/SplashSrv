use bitflags::bitflags;
use deku::prelude::*;
use serde::{Deserialize, Serialize};

use self::helpers::{AString, WString};
use crate::data::record::{GCRecord, GHRecord};
use crate::data::{
    record::{CRecord, URecord},
    report::GameReport,
    Appearance, Class, CountedItem, Item, ParamTuple, Rank, SellCaddy, SellItem,
};

mod helpers;

pub type UID = i32;
pub type CID = i32;
pub type ChrUID = i32;
pub type LobbyNum = i8;
pub type RoomNum = i8;

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct Stat: u32 {
        const READY = 1;
        const EXIT = 2;
        const GALLERY = 4;
        const ROUND = 8;
        const AFK = 0x10;
        const BUSY = 0x20;
        const STEALTH_1 = 0x40;
        const STEALTH_2 = 0x80;
        const S100 = 0x100;
        const S200 = 0x200;
        const S400 = 0x400;
        const S800 = 0x800;
        const S1000 = 0x1000;
        const S2000 = 0x2000;
        const S4000 = 0x4000;
        const S8000 = 0x8000;
    }
}

impl DekuRead<'_> for Stat {
    fn read(
        input: &deku::bitvec::BitSlice<u8, deku::bitvec::Msb0>,
        ctx: (),
    ) -> Result<(&deku::bitvec::BitSlice<u8, deku::bitvec::Msb0>, Self), DekuError>
    where
        Self: Sized,
    {
        let (rest, val) = u32::read(input, ctx)?;
        let val = Stat::from_bits_retain(val);
        Ok((rest, val))
    }
}

impl DekuWrite for Stat {
    fn write(
        &self,
        output: &mut deku::bitvec::BitVec<u8, deku::bitvec::Msb0>,
        ctx: (),
    ) -> Result<(), DekuError> {
        self.bits().write(output, ctx)
    }
}

#[derive(Debug, Clone, DekuRead, DekuWrite)]
pub struct DateTime {
    pub year: i16,
    pub month: i8,
    pub day: i8,
    pub hour: i8,
    pub minute: i8,
    pub second: i8,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, DekuRead, DekuWrite)]
#[deku(type = "i8")]
pub enum Mode {
    None = -1,
    Main = 0,
    VS = 1,
    Competition = 2,
    Quick = 3,
    Mode4 = 4,
    Single = 5,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, DekuRead, DekuWrite)]
#[deku(type = "i8")]
pub enum Status {
    Err = -1,
    OK = 0,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, DekuRead, DekuWrite)]
#[deku(type = "i8")]
pub enum FriendRequestStatus {
    Accept = 1,
    Deny = 0,
}

#[derive(Debug, DekuRead, DekuWrite)]
pub struct PacketHeader {
    pub id: i16,
    pub pid: i16,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, DekuRead, DekuWrite)]
#[deku(ctx = "my_id: i16", id = "my_id")]
pub enum Packet {
    // Packet 0: empty handler

    // Client
    #[deku(id = "1")]
    SEND_IDPASS(IDPass),

    // Server
    #[deku(id = "2")]
    ACK_IDPASS(AckIDPassResult),

    // Client
    #[deku(id = "3")]
    REQ_GMSVLIST,

    // Server
    #[deku(id = "4")]
    SEND_GMSVDATA(GmsvData),

    // Server
    #[deku(id = "5")]
    ACK_GMSVLIST,

    // Client
    // NOTE: uses pid=252 if switching servers?
    #[deku(id = "6")]
    SEND_IDPASS_G(IDPass),

    // Server
    #[deku(id = "7")]
    ACK_IDPASS_G(UData),

    // Client
    #[deku(id = "8")]
    REQ_CHG_MODE(Mode),

    // Server
    #[deku(id = "9")]
    ACK_CHG_MODE(Mode),

    // Client
    #[deku(id = "10")]
    GET_LOBBY_NUM,

    // Server
    #[deku(id = "11")]
    SEND_LOBBY_NUM(i8),

    // Client - PID matters
    #[deku(id = "12")]
    GET_LOBBY_DATA { index: i8, mode: Mode },

    // Server
    #[deku(id = "13")]
    SEND_LOBBY_DATA(LobbyData),

    // Client - Enter lobby
    #[deku(id = "14")]
    REQ_ENTER_LOBBY(LobbyNum),

    // Server
    #[deku(id = "15")]
    ACK_ENTER_LOBBY(LobbyNum),

    // Client - Make room (PID important)
    #[deku(id = "16")]
    REQ_MAKE_ROOM(Packet19),

    // Server
    #[deku(id = "17")]
    ACK_MAKE_ROOM(RoomNum),

    // Client - Room list related
    #[deku(id = "18")]
    GET_ROOMS,

    // Server - room information
    #[deku(id = "19")]
    PKT_19(Packet19),

    // Client - Enter room
    #[deku(id = "20")]
    REQ_ENTER_ROOM {
        room: RoomNum,
        #[deku(bits = 1, pad_bits_after = "31")]
        unk_room_flag: bool,
        room_password: WString<17>,
    },

    // Server
    #[deku(id = "21")]
    ACK_ENTER_ROOM(Packet19),

    // Client - Request UList for room (PID important)
    #[deku(id = "22")]
    REQ_ULIST(Mode, LobbyNum, RoomNum),

    // Server
    #[deku(id = "23")]
    SEND_ULIST(UList),

    // Client - Exit room (PID important)
    #[deku(id = "24")]
    PKT_24,

    // Server
    #[deku(id = "25")]
    ACK_EXIT_ROOM(Status),

    // Client/Server - Send my USTAT
    #[deku(id = "26")]
    SEND_USTAT { cid: CID, uid: UID, stat: Stat },

    // Client/Server - Chat
    #[deku(id = "27")]
    SEND_MESSAGE {
        cid: CID,
        // 0: cid=-1, "ALL"
        // 1: cid=-1, name is set
        // 3: cid=-1, "CIRCLE_ALL"
        msg_type: i8, // 0,1,2,3,4
        server_id: i8,
        name: WString<19>,
        len: i16,
        #[deku(count = "len")]
        message: Vec<u16>,
    },

    // Client - Update room (PID important)
    #[deku(id = "28")]
    PKT_28(Packet19),

    // Server - Update room ack
    #[deku(id = "29")]
    PKT_29(Status),

    // Server - Room stat
    #[deku(id = "30")]
    PKT_30(RoomStat),

    // Client - Request game start
    #[deku(id = "31")]
    REQ_GAMESTART,

    // Server - Game start
    #[deku(id = "32")]
    ORD_GAMESTART {
        mode: Mode,
        rule: i8,
        time: u8,
        member: i8,
        member_max: i8,
        course: i8,
        season: i8,
        holes: i8,
        hole_no: [i8; 18],
        wind_dir: [i8; 18],
        wind_pow: [i8; 18],
        weather: [i8; 18],
        cup_pos: [i8; 18],
        cid: [CID; 50],
        caddies: [i16; 50],
        caddie_reliance: [i32; 50],
        ball_array: [i32; 50], // is this ItemID?
        hold_box: [[CountedItem; 8]; 50],
    },

    // Client - Send shot club
    #[deku(id = "33")]
    CLIENT_CRCLUB(i8),

    // Server
    #[deku(id = "34")]
    SEND_CRCLUB {
        cid: CID,
        club: i8, // ranges from 0 to 14
    },

    // Client - Send shot dir
    #[deku(id = "35")]
    CLIENT_DIRECTION(f32),

    // Server
    #[deku(id = "36")]
    SEND_DIRECTION { cid: CID, dir: f32 },

    // Client - Send shot stuff
    #[deku(id = "37")]
    CLIENT_SHOT {
        clock: u64,
        server_cid: CID, // not set by client
        dir: f32,
        power: i16,
        impact: i16,
        hit_x: i8,
        hit_y: i8,
        // club: -1 = timeout, -2 = whiff
        // has flags 0x10, 0x20, 0x40 depending on shot type
        club: i8,
    },

    // Server
    #[deku(id = "38")]
    SEND_SHOT {
        clock: u64,
        cid: CID,
        dir: f32,
        power: i16,
        impact: i16,
        hit_x: i8,
        hit_y: i8,
        // club: -1 = timeout, -2 = whiff
        // has flags 0x10, 0x20, 0x40 depending on shot type
        club: i8,
    },

    // Client - SendScore
    #[deku(id = "39")]
    SEND_SCORE(GameReport),

    // Client - Request URecord (PID important, UID can be -1 sometimes, CharDetails)
    #[deku(id = "40")]
    REQ_URECORD(UID),

    // Server - SEND_URECORD
    #[deku(id = "41")]
    SEND_URECORD {
        uid: UID,
        data: URecord,
        status: Status,
    },

    // Client - Request CRecord (PID important, CharDetails)
    #[deku(id = "42")]
    REQ_CRECORD {
        uid: UID,
        course: i8,
        season: i8,
        hole_idx: i8, // 0=3H 1=6H 2=9H 3=18H
    },

    // Server - SEND_CRECORD
    #[deku(id = "43")]
    SEND_CRECORD {
        uid: UID,
        course: i8,
        season: i8,
        hole_idx: i8,
        data: CRecord,
        status: Status,
    },

    // Client - Send loadstat
    #[deku(id = "44")]
    CLIENT_LOADSTAT(i8),

    // Server
    #[deku(id = "45")]
    SEND_LOADSTAT(CID, i8),

    // Client - Send Ballpos
    #[deku(id = "46")]
    CLIENT_BALLPOS {
        server_cid: CID, // not always set by client
        hole: i8,
        stat: i8,
        x: f32,
        y: f32,
        z: f32,
    },

    // Server
    #[deku(id = "47")]
    SEND_BALLPOS {
        cid: CID,
        hole: i8,
        stat: i8,
        x: f32,
        y: f32,
        z: f32,
    },

    // Client - Send Holeout
    #[deku(id = "48")]
    CLIENT_HOLEOUT {
        server_cid: CID, // not set by client
        hole: i8,
        score: i8,
        gp: i16,
    },

    // Server
    #[deku(id = "49")]
    SEND_HOLEOUT {
        cid: CID,
        hole: i8,
        score: i8,
        gp: i16,
    },

    // Client - also quick matching related? sub_687EB0
    // sent when marking yourself as ready
    #[deku(id = "50")]
    REQ_ADD_RANKMEMBER(Packet50Data),

    // Server
    #[deku(id = "51")]
    ACK_ADD_RANKMEMBER(Status),

    // Client - quick matching related?
    // sent when cancelling your readiness
    #[deku(id = "52")]
    REQ_RMV_RANKMEMBER,

    // Server
    #[deku(id = "53")]
    ACK_RMV_RANKMEMBER(Status),

    // Server
    // sent when a match is completed after becoming ready, if you're supposed to move servers
    // MyRoomTask goes into phase 9
    // you might also get packet 210 after this, i think
    #[deku(id = "54")]
    ORD_RANKJUMP { sv_no: i8, vsplayer_uid: UID },

    // Client - sent by MyRoom in phase 10, param is -1 or 0
    // means you're done moving servers after a RANKJUMP?
    #[deku(id = "55")]
    PKT_55,

    // Client - ReqStartRank also quick matching related? sub_68D190
    #[deku(id = "56")]
    PKT_56(Packet50Data),

    // Server
    #[deku(id = "57")]
    SEND_RANKDATA {
        uid: UID,
        score: i16,
        win_combo: u8,
        #[deku(bits = 5)]
        rank: u8,
        #[deku(bits = 1)]
        rank_up: bool,
        #[deku(bits = 1, pad_bits_after = "1")]
        rank_down: bool,
    },

    // 58 - client unused
    // 59 - server unused
    // 60 - client unused
    // 61 - server unused
    // 62 - client unused
    // 63 - server unused
    // 64 - server unused

    // Client - sent by ChatTask to get friends list
    #[deku(id = "65")]
    PKT_65(UID),

    // Server - info about a user?
    #[deku(id = "66")]
    PKT_66 {
        cid: CID,
        uid: UID,
        stat: i16,
        unk: [u8; 15],
        name: WString<17>,
    },

    // Client - sent by DeliveryTask, maybe to look up player?
    #[deku(id = "67")]
    PKT_67 {
        unk1: i32,
        unk2: i32,
        name: WString<19>,
    },

    // Server - reply to 67
    #[deku(id = "68")]
    PKT_68 {
        unk1: i32,
        uid: UID, // negative if invalid
        name: WString<19>,
    },

    // Client - send friend request
    #[deku(id = "69")]
    PKT_69(UID),

    // Server - ack friend request
    #[deku(id = "70")]
    PKT_70(UID, Status),

    // Client - PID important
    #[deku(id = "71")]
    REQ_FRIENDS,

    // Server - friend info
    #[deku(id = "72")]
    PKT_72 {
        count: i32,
        #[deku(count = "count")]
        users: Vec<UID>,
    },

    // Client - PID important
    #[deku(id = "73")]
    REQ_INBOUND_REQUESTS,

    // Server - inbound friend requests
    #[deku(id = "74")]
    PKT_74 {
        count: i32,
        #[deku(count = "count")]
        users: Vec<UID>,
    },

    // Client - PID important
    #[deku(id = "75")]
    REQ_OUTBOUND_REQUESTS,

    // Server - outbound friend requests
    #[deku(id = "76")]
    PKT_76 {
        count: i32,
        #[deku(count = "count")]
        users: Vec<UID>,
    },

    // Client - accept/deny friend request
    #[deku(id = "77")]
    PKT_77(UID, FriendRequestStatus),

    // Server - friend action
    #[deku(id = "78")]
    PKT_78(UID, Status, FriendRequestStatus),

    // Client - remove friend
    #[deku(id = "79")]
    PKT_79(UID),

    // Server - remove friend
    #[deku(id = "80")]
    PKT_80(UID, Status),

    // Client - cancel friend request
    #[deku(id = "81")]
    PKT_81(UID),

    // Server - remove friend request
    #[deku(id = "82")]
    PKT_82(UID, Status),

    // Client - handled by ClsF48, CharDetails? PID important
    #[deku(id = "83")]
    REQ_APPEAR(CID),

    // Server
    #[deku(id = "84")]
    SEND_APPEAR(CID, i32, Appearance),

    // Client - ???
    #[deku(id = "85")]
    PKT_85 {
        server_cid: CID,
        unk: f32,
        x: f32,
        y: f32,
        z: f32,
    },

    // Server
    #[deku(id = "86")]
    SEND_CHRPOS {
        cid: CID,
        unk: f32,
        x: f32,
        y: f32,
        z: f32,
    },

    // Client - PID important
    #[deku(id = "87")]
    REQ_ULIST_L(Mode, LobbyNum),

    // Server
    #[deku(id = "88")]
    SEND_ULIST_L(UListL),

    // Client - Get sell item list
    #[deku(id = "89")]
    PKT_89,

    // Server
    #[deku(id = "90")]
    SEND_SELLITEMLIST {
        count: i16,
        #[deku(count = "count")]
        items: Vec<SellItem>,
    },

    // Client - ReqItemBuy one type
    #[deku(id = "91")]
    REQ_BUY_ITEM(CountedItem),

    // Server
    #[deku(id = "92")]
    ACK_BUY_ITEM(BuyItemResult),

    // Client - CharDetails, PID important
    #[deku(id = "93")]
    PKT_93,

    // Server
    #[deku(id = "94")]
    REP_MONEY { gp: i32, sc: i32 },

    // Client
    #[deku(id = "95")]
    SET_FIRST_CHARACTER_APPEARANCE(Appearance),

    // Server
    #[deku(id = "96")]
    ACK_FIRST_CHARACTER_APPEARANCE(Status),

    // Client - Sent by RankingTask and PlayerList and CharDetails, PID important
    #[deku(id = "97")]
    PKT_97(UID),

    // Client - Sent by CharacterDetailsTask
    #[deku(id = "98")]
    PKT_98(CID),

    // Server
    #[deku(id = "99")]
    SEND_CHRUID {
        count: i32,
        // are these part of the same array...?
        cid: CID,
        #[deku(count = "count - 1")]
        chr_uids: Vec<ChrUID>,
    },

    // Client - ReqChrData (PlayerInfo associatedPIDs[1])
    #[deku(id = "100")]
    REQ_CHRDATA { cid: CID, chr_uid: ChrUID },

    // Server
    #[deku(id = "101")]
    SEND_CHRDATA { cid: CID, uid: UID, data: ChrData },

    // Client - Get_ChrData (used by MyRoomTask)
    // Gets all the characters owned by this user
    #[deku(id = "102")]
    GET_CHRDATA(CID),

    // Client
    #[deku(id = "103")]
    REQ_CHG_APPEAR {
        cid: CID,
        chr_uid: ChrUID,
        appear: Appearance,
    },

    // Server reply to REQ_CHG_APPEAR, wtf is up with the naming
    #[deku(id = "104")]
    PKT_104(Status),

    // Client
    #[deku(id = "105")]
    SET_PLAYER_NAME(SetPlayerName),

    // Server
    #[deku(id = "106")]
    ACK_SET_CHARACTER_NAME(Status),

    // Client - called by CourceRecordTask_vf34
    #[deku(id = "107")]
    PKT_107 {
        course: i8,
        season: i8,
        // -1, 0, 1, 2
        unk: i8,
    },

    // Server
    #[deku(id = "108")]
    PKT_108 {
        gcrecord: GCRecord,
        ghrecord: [GHRecord; 18],
    },

    // Client - called by MailTask
    #[deku(id = "109")]
    REQ_UNRECEIVE_SMAIL_CNT(UID),

    // Server - UnreceiveSmailCnt
    #[deku(id = "110")]
    SEND_UNRECEIVE_SMAIL_CNT { uid: UID, cnt: i8 },

    // Client - MailTask before setting phase 4
    // only gets called if the count from 110 is non-zero
    #[deku(id = "111")]
    PKT_111(UID),

    // Server - MailTask vf40
    #[deku(id = "112")]
    PKT_112 {
        unk1: i32,
        unk2: i32,
        cnt: i32,
        #[deku(count = "cnt")]
        values: Vec<i32>,
    },

    // Client - MailTask
    // Fetches a mail based on the ID returned in the 112 array
    #[deku(id = "113")]
    PKT_113(UID, i32),

    // Server - MailTask vf44 incoming mail
    #[deku(id = "114")]
    PKT_114 {
        mail_uid: UID,
        from_uid: i32,
        to_uid: i32,
        date_time: DateTime,
        len: i16,
        #[deku(count = "len")]
        utf8_text: Vec<u8>,
    },

    // Server - MailTask outgoing mail
    #[deku(id = "115")]
    PKT_115 {
        mail_uid: UID, // not filled by client
        from_uid: i32, // not filled by client
        to_uid: i32,
        date_time: DateTime, // not filled by client
        len: i16,
        #[deku(count = "len")]
        utf8_text: Vec<u8>,
    },

    // Server - MailTask vf46
    // this is just a result
    #[deku(id = "116")]
    PKT_116(MailSendResult),

    // Client - PID important
    #[deku(id = "117")]
    REQ_BLOCKLIST(i32),

    // Server - blocklist
    #[deku(id = "118")]
    PKT_118 {
        unk1: i32,
        unk2: i32,
        count: i32,
        #[deku(count = "count")]
        users: Vec<UID>,
    },

    // Client - block user
    #[deku(id = "119")]
    PKT_119(UID),

    // Server - block user
    #[deku(id = "120")]
    PKT_120(UID, Status),

    // Client - unblock user
    #[deku(id = "121")]
    PKT_121(UID),

    // Server - block user
    #[deku(id = "122")]
    PKT_122(UID, Status),

    // Client - do player search, PID may matter
    #[deku(id = "123")]
    PKT_123 {
        name: WString<19>,
        unk1: i8,
        unk2: i8,
        // only the bottom 2 bits of these are used
        flags: u32,
    },

    // Server
    #[deku(id = "124")]
    SEND_SEARCH_USER { sv_no: i8, ulist: UList },

    // Client - stat update again?
    #[deku(id = "125")]
    PKT_125 {
        uid: UID,  // sometimes -1
        stat: u32, // GlobalInfo::myStat & 0x7fffffff
    },

    // Client - Cup In
    #[deku(id = "126")]
    CLIENT_CUP_IN { hole: i8, score: i8 },

    // Server
    #[deku(id = "127")]
    REP_ITEMDROP { cid: CID, val: i8 },

    // Client/Server - game timer / player timer
    #[deku(id = "128")]
    REP_CLOCK {
        timer: i64,
        unk: i32, // CID minus 1?
    },

    // Client - do room search, PID may matter
    #[deku(id = "129")]
    PKT_129 {
        gmsv_no: i8,
        unk1: i8,
        unk2: i8,
        unk3: i8,
        bitfield: u32,
    },

    // Server
    #[deku(id = "130")]
    SEND_SEARCH_ROOM { sv_no: i8, data: Packet19 },

    // Client - CharDetails. Arg is always -1
    #[deku(id = "131")]
    PKT_131(i32),

    // Server - items you have?
    #[deku(id = "132")]
    PKT_132 {
        count: i32,
        #[deku(count = "count")]
        items: Vec<CountedItem>,
    },

    // Client - CharDetails. Arg is always -1
    #[deku(id = "133")]
    PKT_133(i32),

    // Server/Client - golfbag contents? (someItemIDsShifted)
    #[deku(id = "134")]
    PKT_134 {
        x4: i32,
        cid: CID,
        items: [Item; 8],
        unk: [u8; 4060],
    },

    // Client - Send command
    #[deku(id = "135")]
    CLIENT_PCOMMAND {
        server_cid: CID, // not set by client
        p0: u32,
        p1: u32,
        #[deku(pad_bytes_after = "2")]
        cmd_and_flag: u16,
    },

    // Server
    #[deku(id = "136")]
    SEND_PCOMMAND {
        cid: CID,
        p0: u32,
        p1: u32,
        #[deku(pad_bytes_after = "2")]
        cmd_and_flag: u16,
    },

    // Client - Req CurrChrUID (PID important - PlayerInfo associatedPIDs[0], CharDetails)
    #[deku(id = "137")]
    PKT_137(CID),

    // Client - PID important
    #[deku(id = "138")]
    REQ_CHG_CRCHRUID(ChrUID),

    // Server
    #[deku(id = "139")]
    SEND_CRCHRUID { cid: CID, now_chr_uid: ChrUID },

    // Server
    #[deku(id = "140")]
    SEND_GROW_PARAM {
        a: i16,
        master_point: i32,
        p0_a: i16,
        p1_a: i16,
        p2_a: i16,
        p3_a: i16,
        p0_b: i16,
        p1_b: i16,
        p2_b: i16,
        p3_b: i16,
        caddie_point: i16,
        extra_bonus_value: i32,
    },

    // Client - msg 1004 and 100E on 4001:0101
    #[deku(id = "141")]
    PKT_141,

    // Client unused 142
    // Client unused 143
    // Client unused 144

    // Client
    #[deku(id = "145")]
    REQ_CHG_CHR_PARAM {
        chr_uid: ChrUID,
        cr_class: Class,
        power: i32,  // always -1??
        impact: i32, // always -1??
        params: [ParamTuple; 8],
        club: Item,
        ball: Item,
        caddie: Item,
    },

    // Server - badly named REQ_CHG_CHR_PARAM ack
    #[deku(id = "146")]
    ACK_CHG_CHR_PARAM(Status),

    // Client - Get available caddies
    #[deku(id = "147")]
    PKT_147,

    // Server
    #[deku(id = "148")]
    SEND_SELL_CADDIE_LIST {
        count: i16,
        #[deku(count = "count")]
        items: Vec<SellCaddy>,
    },

    // Client - Sent by DeliveryTask_Execute
    // arg is always -1?
    #[deku(id = "149")]
    PKT_149(i32),

    // Server
    #[deku(id = "150")]
    SEND_DELIVER(Delivery),

    // Client - ReqCaddieEmploy
    // Quantity in CountedItem represents the time:
    // 0 = 3 hour, 1 = 3 day, 2 = 30 day, 3 = infinity, 4 = time error
    #[deku(id = "151")]
    PKT_151(CountedItem),

    // Server
    #[deku(id = "152")]
    ACK_EMPLOY_CADDIE(BuyItemResult),

    // Client - CharDetails. Arg might be UID but need to confirm, is sometimes -1. PID matters
    #[deku(id = "153")]
    PKT_153(i32),

    // Server
    #[deku(id = "154")]
    PKT_154(CaddieData),

    // Client - Item Use
    // arg is 0 or 1?
    #[deku(id = "155")]
    PKT_155(Item, i8),

    // Server - Item Use
    #[deku(id = "156")]
    ACK_USE_ITEM(Item, i32),

    // Server - Item Use 2
    #[deku(id = "157")]
    REP_USE_ITEM(CID, Item, i32),

    // Client - Send delivery
    #[deku(id = "158")]
    PKT_158(Delivery),

    // Server
    #[deku(id = "159")]
    ACK_SEND_DELIVER(i8, SendDeliverResult),

    // Client - Another delivery thing phase 0x703
    #[deku(id = "160")]
    PKT_160 {
        delivery: Delivery,
        unk4: i8, // 1, 3, 4, 5, 6 depending on stuff
    },

    // Server
    // not sure if this is the same result type
    #[deku(id = "161")]
    ACK_ANS_DELIVER {
        index: i8,
        unk4: i8, // i think this is the same as 160 unk4
        result: SendDeliverResult,
    },

    // Client - Probably gets macro data? Arg is always -1
    #[deku(id = "162")]
    PKT_162(i32),

    // Server - Receive macro data
    #[deku(id = "163")]
    PKT_163 { which: i8, text: AString<65> },

    // Client - Send macro
    #[deku(id = "164")]
    PKT_164 { which: i8, text: AString<65> },

    // Server - Macro saved
    #[deku(id = "165")]
    PKT_165 {
        which: i8,      // assumed
        status: Status, // assumed
    },

    // Client - Get salon item list
    #[deku(id = "166")]
    PKT_166,

    // Server
    #[deku(id = "167")]
    SEND_SALON_ITEM_LIST {
        count: i16,
        #[deku(count = "count")]
        items: Vec<SellItem>,
    },

    // Client - ReqSalonItemBuy
    #[deku(id = "168")]
    PKT_168(CountedItem),

    // Server
    #[deku(id = "169")]
    ACK_BUY_SALON_ITEM(BuyItemResult),

    // Client - Send titles. CharDetails
    #[deku(id = "170")]
    PKT_170,

    // Server
    #[deku(id = "171")]
    SEND_TITLES(UID, u128),

    // Client - GetTitle - assigns a title as obtained?
    #[deku(id = "172")]
    PKT_172(i16),

    // Server
    #[deku(id = "173")]
    ACK_GET_TITLE(Status),

    // Client - Change title
    #[deku(id = "174")]
    REQ_CHG_TITLE(i16),

    // Server
    #[deku(id = "175")]
    ACK_CHG_TITLE(Status),

    // Client - Send Telop
    #[deku(id = "176")]
    PKT_176 {
        id: i32,
        arg1: i32,
        arg2: i32,
        arg3: i32,
    },

    // Server - Send Telop
    #[deku(id = "177")]
    SEND_TELOP {
        id: i32,
        arg1: i32,
        arg2: i32,
        arg3: i32,
    },

    // Server
    #[deku(id = "178")]
    REP_COMPRES { cid: [CID; 20], count: [i32; 20] },

    // Client - Related to CompeLounge
    #[deku(id = "179")]
    PKT_179,

    // Client - ReqUdata (PlayerInfo associatedPIDs[2], CharDetails)
    #[deku(id = "180")]
    REQ_UDATA(UID),

    // Server
    #[deku(id = "181")]
    PKT_181(UData),

    // Client - Request ranking
    #[deku(id = "182")]
    PKT_182 {
        // this is a massive soup of things
        start: i32,
        mode: i8,
        submode: i8,
        submode2: i8,
    },

    // Server
    #[deku(id = "183")]
    PKT_183 {
        count: i8,
        #[deku(count = "count")]
        entries: Vec<Packet183Entry>,
    },

    // Server - 184 unused

    // Client - Send loadstat 2
    #[deku(id = "185")]
    CLIENT_LOADSTAT2(i8),

    // Server - Sync loadstat 2
    #[deku(id = "186")]
    SEND_LOADSTAT2(CID, i8),

    // Server - nothing in here other than a log?
    #[deku(id = "187")]
    ACK_GAMESTART(Status),

    // Client - 188 unused

    // Client - Chg Holdbox?
    #[deku(id = "189")]
    PKT_189 { hold_item: [Item; 8] },

    // Server
    #[deku(id = "190")]
    ACK_CHG_HOLDBOX(Status),

    // Server - adds item to inventory
    #[deku(id = "191")]
    SEND_DROPITEM([CountedItem; 10]),

    // Client - GameCenterTask, CodeCenterTask
    // arg always -1 for GC or 0 for CC?
    // PID matters
    #[deku(id = "192")]
    PKT_192(i32),

    // Server - num items in delivery box?
    #[deku(id = "193")]
    PKT_193 {
        unk: i32, // -3 = error, 0 = ok?
        num_items: i32,
    },

    // Client - Send command 2
    #[deku(id = "194")]
    PKT_194 {
        server_cid: CID, // not set by client
        p0: u32,
        p1: u32,
        #[deku(pad_bytes_after = "2")]
        cmd_and_flag: u16,
        p2: f32,
        p3: f32,
    },

    // Server
    #[deku(id = "195")]
    SEND_PCOMMAND2 {
        cid: CID,
        p0: u32,
        p1: u32,
        #[deku(pad_bytes_after = "2")]
        cmd_and_flag: u16,
        p2: f32,
        p3: f32,
    },

    // Client - ReqItemBuy_ByTicket
    #[deku(id = "196")]
    PKT_196 {
        // what ticket to use when buying the item
        ticket: CountedItem,
        // what item you want to buy
        item: CountedItem,
    },

    // Server
    #[deku(id = "197")]
    ACK_BUY_ITEM_BY_TICKET(BuyItemResult),

    // Client - GameCenterTask play UFO game (1 = decrement coin, 2 = decrement 1/10 coin)
    #[deku(id = "198")]
    PKT_198(i8),

    // Server - GameCenter outcome of UFO game
    #[deku(id = "199")]
    PKT_199 {
        item: CountedItem,
        // -3: failed to connect to server
        // -2: delivery box is full
        // -1: not enough coins
        // 0: ok
        result: i8,
        // 0=?, 1-?, 2-?, 3-?, 4-?
        outcome: i8,
    },

    // Client - ReqCaddieEmploy_ByTicket
    #[deku(id = "200")]
    PKT_200 {
        // what ticket to use when buying the item
        ticket: CountedItem,
        // Quantity represents the duration
        // 0 = 3 hour, 1 = 3 day, 2 = 30 day, 3 = infinity, 4 = time error
        item: CountedItem,
    },

    // Server
    #[deku(id = "201")]
    ACK_EMPLOY_CADDIE_BY_TICKET(BuyItemResult),

    // Client - ReqSalonItemBuy_ByTicket
    #[deku(id = "202")]
    PKT_202 {
        // what ticket to use when buying the item
        ticket: CountedItem,
        // what item you want to buy
        item: CountedItem,
    },

    // Server
    #[deku(id = "203")]
    ACK_BUY_SALON_ITEM_BY_TICKET(BuyItemResult),

    // Client - CharDetails. Might be UID but not sure
    #[deku(id = "204")]
    PKT_204(i32),

    // Server
    #[deku(id = "205")]
    SEND_NP { uid: UID, sp: i32 },

    // 206 is missing

    // Server
    #[deku(id = "207")]
    ACK_ADD_NP,

    // Client - ReqItemBuy another type
    #[deku(id = "208")]
    PKT_208(CountedItem),

    // Server
    #[deku(id = "209")]
    ACK_BUY_ITEM_BY_NP(BuyItemResult),

    // Server
    // I think this is used when the server has picked an opponent in quick matching
    // MyRoomTask goes into phase 12
    #[deku(id = "210")]
    SEND_RANK_EDATA { char_type: i8, rank: i8 },

    // Client - Set team from VS or Compe Lounge
    #[deku(id = "211")]
    PKT_211(i8),

    // Server
    #[deku(id = "212")]
    SEND_SET_TEAM(CID, i8),

    // Client - GameCenterTask play slots game (3 = decrement medal, 12 = decrement daily plays)
    #[deku(id = "213")]
    PKT_213(i8),

    // Server - GameCenter outcome of slots game
    #[deku(id = "214")]
    PKT_214 {
        item: CountedItem,
        // -3: failed to connect to server
        // -2: delivery box is full
        // -1: not enough medals
        // 0: ok
        result: i8,
        // 0=hit0, 1=hit1, 2=hit2, 3=hit2, 4=miss
        outcome: i8,
    },

    // Client - set quick settings itemon
    #[deku(id = "215")]
    PKT_215(i8),

    // Client/Server - Request to transfer owner to somebody?
    #[deku(id = "216")]
    REQ_CHG_OWNER(CID),

    // Client/Server - Accept/deny owner transfer?
    #[deku(id = "217")]
    PKT_217(i8),

    // Server
    #[deku(id = "218")]
    SEND_CHG_OWNER(CID),

    // Client - kick user from room?
    #[deku(id = "219")]
    PKT_219(CID),

    // Server
    #[deku(id = "220")]
    PKT_220(Status),

    // Server
    #[deku(id = "221")]
    SEND_KICK_MEMBER(CID),

    // Client - ReqChgCaddieByItem
    #[deku(id = "222")]
    PKT_222 { item: Item, which_caddie: u32 },

    // Server - ChangeKun
    #[deku(id = "223")]
    PKT_223(Status),

    // Server - ChangeCaddie
    #[deku(id = "224")]
    PKT_224 {
        cid: CID,
        caddie_type: i32,
        unk: i32,
    },

    // Server
    #[deku(id = "225")]
    SEND_CHG_TITLE { uid: UID, title: i32 },

    // Server
    #[deku(id = "226")]
    SEND_CHG_UDATA {
        uid: UID,
        chg: ChgUDataType,
        p0: UID,
        p1: i32,
    },

    // Client - GameCenterTask get number of plays?
    #[deku(id = "227")]
    PKT_227,

    // Server - Return number of plays remaining today
    #[deku(id = "228")]
    PKT_228(i8),

    // Client - Ping
    #[deku(id = "229")]
    PKT_229,

    // Server - Ping reply
    #[deku(id = "230")]
    PKT_230(i64, i16),

    // Server - Enable caddie list
    #[deku(id = "231")]
    PKT_231 {
        count: i32,
        #[deku(count = "count")]
        list: Vec<i32>,
    },

    // Client - Update game options
    #[deku(id = "232")]
    PKT_232 {
        unk_neg1: i32,
        bitfield: u8, // matches StuffF9::F4
    },

    // Server - no client handling other than a debug msg
    #[deku(id = "233")]
    SEND_CHG_UDATA_FLAG { unk: i32, status: Status },

    // Client - Send Stop Ballpos
    #[deku(id = "234")]
    CLIENT_STOP_BALLPOS {
        server_cid: CID, // not set by client
        hole: i8,
        stat: i8,
        x: f32,
        y: f32,
        z: f32,
    },

    // Server
    #[deku(id = "235")]
    SEND_STOP_BALLPOS {
        cid: CID,
        hole: i8,
        stat: i8,
        x: f32,
        y: f32,
        z: f32,
    },

    // Server
    #[deku(id = "236")]
    ORD_COLOR_RESULT {
        // this is None if the player has not just been assigned an element on this round
        element: Element,
        // this signifies what occurred in the last cycle
        last_element: Element,
        color_result: i8,
        rank_in_color: i32, // what rank was this in the colour?
        gp: i32,
        item: CountedItem,
    },

    // Server
    #[deku(id = "237")]
    SEND_MP_TABLE {
        count: i32,
        #[deku(count = "count")]
        table: Vec<i32>,
    },

    // Client - sub_736570
    #[deku(id = "238")]
    REQ_ADD_GP {
        unk: i32, // set to -1, this is probably an ID of some kind
        result_gp: i32,
    },

    // Server
    #[deku(id = "239")]
    ACK_ADD_GP {
        inum0: i32, // figure me out
        inum1: i32, // figure me out
    },

    // Client - CMRoomListTask_Execute - PID matters?
    // This might refresh the data for a room
    #[deku(id = "240")]
    PKT_240 {
        // !! stored as 32 bits rather than 8 bits !!
        mode: u32,
        lobby: u32,
        room: u32,
    },

    // Client - CaddieItemRecoveryOB_Task ItemUseRequest
    #[deku(id = "241")]
    PKT_241(Item),

    // Server
    #[deku(id = "242")]
    ACK_USE_HOLDITEM {
        item_id: Item,
        new_count: i32, // can be -1 for fail
    },

    // 243 - unused
    // 244 - unused
    // 245 - unused

    // Client - Return?
    #[deku(id = "246")]
    PKT_246,

    // Server
    #[deku(id = "247")]
    REP_RETURN_LOUNGE_ALL,

    // 248 - unused
    // 249 - unused

    // Client/Server - Ping from DisconCheckTask
    #[deku(id = "250")]
    REQ_PING(i32),

    // Client/Server - Ping reply (PID important too)
    #[deku(id = "251")]
    PKT_251(i32),

    // 252 - unused
    // 253 - unused
    // 254 - unused
    // 255 - unused
    #[deku(id = "256")]
    SEND_COMP_ITEM {
        count: i32,
        #[deku(count = "count")]
        items: Vec<CountedItem>,
    },

    // 257 - unused
    // 258 - server unused
    // 259 - unused
    // 260 - server unused
    // 261 - server unused
    // 262 - unused

    // Client - Init recycling system? (sent when recycling shop opened)
    #[deku(id = "263")]
    PKT_263,

    // Server - RecycleTaskZ vf04 - info about recyclables?
    #[deku(id = "264")]
    PKT_264 {
        count: i16,
        #[deku(count = "count")]
        items: Vec<[u8; 0x1C]>,
    },

    // Server - RecycleTaskZ vf08
    #[deku(id = "265")]
    PKT_265,

    // Client - RecycleTaskZ vf0C - start recycling using an Eco Ticket
    #[deku(id = "266")]
    PKT_266 {
        index: i16, // someIndex from packet 264 item struct
        // TODO: is this bit mapping correct?
        #[deku(bits = 1, pad_bits_before = "7", pad_bits_after = "8")]
        is_gold_ticket: bool,
    },

    // Server - RecycleTaskZ vf10 - recycle result
    // 0 = recycle succeeded, -1 = recycle failed
    // there are some other errors i guess
    // On receipt, client reduces your amount of tickets by 1 and decrements all 5 materials
    #[deku(id = "267")]
    PKT_267(i8),

    // Client
    #[deku(id = "268")]
    GET_MODECTRL,

    // Server
    #[deku(id = "269")]
    SEND_MODECTRL(ModeCtrl),

    // Client - code centre string? PID matters, is 236
    #[deku(id = "270")]
    PKT_270(AString<21>),

    // Server
    #[deku(id = "271")]
    PKT_271 {
        items: [CountedItem; 5],
        status: Status,
        count: i8,
        text: WString<32>,
    },

    // Client - code centre string? PID matters, is 236
    #[deku(id = "272")]
    PKT_272(AString<21>),

    // Server
    #[deku(id = "273")]
    PKT_273(Status),

    // Client - Get item counts for single mode?
    #[deku(id = "274")]
    PKT_274,

    // Server - Set what items you have available in single mode
    #[deku(id = "275")]
    PKT_275 { count: i32, items: [CountedItem; 8] },

    // Client - Trash items
    #[deku(id = "276")]
    PKT_276 {
        count: i32,
        items: [CountedItem; 1024],
    },

    // Server
    #[deku(id = "277")]
    PKT_277(Status),

    // 288 - unused

    // Client - invite user?
    #[deku(id = "279")]
    PKT_279(CID),

    // Server - Ask user if they want to join a room
    #[deku(id = "280")]
    PKT_280 {
        source_uid: UID,
        room_member_uids: [UID; 50],
        room_data: Packet19,
    },

    // Server, delivery related
    #[deku(id = "281")]
    PKT_281,

    // Server - GameGuard auth challenge
    #[deku(id = "282")]
    PKT_282 {
        index: u32,
        val1: u32,
        val2: u32,
        val3: u32,
    },

    // Client - GameGuard auth response
    #[deku(id = "283")]
    PKT_283 {
        index: u32,
        val1: u32,
        val2: u32,
        val3: u32,
    },

    // 284 - unused
    // 285 - unused

    // Client - Player out from gallery? (cRoundMain_SendRetire)
    #[deku(id = "286")]
    PKT_286,

    // 287-301 - unused

    // Server - Ranking Count for RankingTask
    #[deku(id = "302")]
    PKT_302(i32),

    // 303 - unused

    // Server - telop with arbitrary text
    #[deku(id = "304")]
    PKT_304 {
        unk: [u8; 26],
        len: i16,
        #[deku(count = "len")]
        text: Vec<u16>,
    },

    // 305 - unused
    // 306 - unused

    // Server - VS lounge stuff
    #[deku(id = "307")]
    ACK_ULIST_R(Status),

    // Client
    #[deku(id = "308")]
    REQ_SVITEMDATA,

    // Server
    #[deku(id = "309")]
    SEND_SVITEMDATA([SVItemData; 32]),

    // Server
    #[deku(id = "310")]
    ACK_SEND_SVITEMDATA { count: i16 },

    // Client
    #[deku(id = "311")]
    REQ_CLUBDATA,

    // Server
    #[deku(id = "312")]
    SEND_CLUBDATA {
        count: i32,
        #[deku(count = "count")]
        clubdata: Vec<ClubData>,
    },

    // Server
    #[deku(id = "313")]
    REP_END_CLUBDATA { count: i32 },

    // 314 - unused

    // Server - used for unknown/missing feature?
    // there's data in here but it's hard to figure out any sort of structure
    // as the associated code to use it is all stripped
    #[deku(id = "315")]
    PKT_315,

    // Client - Debug message
    #[deku(id = "316")]
    PKT_316 {
        len: i16,
        #[deku(count = "len")]
        message: Vec<u16>,
    },

    #[deku(id_pat = "_")]
    Unknown,
}

#[derive(Debug, DekuRead, DekuWrite)]
pub struct EntirePacket {
    pub header: PacketHeader,
    #[deku(ctx = "header.id")]
    pub packet: Packet,
}

// 1
#[derive(Debug, Clone, DekuRead, DekuWrite)]
pub struct IDPass {
    pub username: AString<17>,
    pub password: AString<17>,
    pub version: u16,
}

// 2
#[derive(Debug, Clone, Copy, Eq, PartialEq, DekuRead, DekuWrite)]
#[deku(type = "i8")]
pub enum AckIDPassResult {
    OK = 0,
    // "Wrong ID or PASS."
    IDError = -1,
    PassError = -2,
    // "The use of this ID is currently suspended."
    BanError = -3,
    // "This account is not valid."
    AccountNotError = -4,
    // "You are already logged in."
    MultiLoginError = -5,
    // "Your version is not the latest."
    VersionError = -6,
}

// 4
#[derive(Debug, Clone, DekuRead, DekuWrite)]
pub struct GmsvData {
    pub number: i16,
    pub ip_address: AString<129>,
    pub port: u16,
    pub enc_key: AString<57>,
    pub name: WString<13>,
    pub comment: WString<13>,
    pub max: i16,
    pub now: i16,
}

#[derive(Debug, Clone, DekuRead, DekuWrite)]
pub struct UData {
    pub cid: CID,
    pub uid: UID,
    pub chr_uid: ChrUID,
    pub golfbag: [Item; 8],
    pub holdbox: [Item; 8],
    pub medals: [[i16; 4]; 4],
    pub awards: [i32; 20],
    pub rank_score_item_on: i16,
    pub rank_score_item_off: i16,
    pub mp: i32,
    pub year: i16,
    pub month: i8,
    pub day: i8,
    pub name: WString<19>,
    pub element: Element,
    pub class: Rank,
    // TODO: I'm 99% sure these bits are not mapped properly by Deku
    // these should all be part of a single u32
    #[deku(bits = 5)]
    pub rank_item_on: u8,
    #[deku(bits = 5)]
    pub rank_item_off: u8,
    #[deku(bits = 5)]
    pub best_rank_item_on: u8,
    #[deku(bits = 5, pad_bits_after = "12")]
    pub best_rank_item_off: u8,
    pub x_f4: u32, //&4 : refuses home delivery
    pub debug: bool,
}

impl Default for UData {
    fn default() -> Self {
        UData {
            cid: 0,
            uid: 0,
            chr_uid: 0,
            golfbag: [Item::default(); 8],
            holdbox: [Item::default(); 8],
            medals: Default::default(),
            awards: Default::default(),
            rank_score_item_on: 0,
            rank_score_item_off: 0,
            mp: 0,
            year: 0,
            month: 0,
            day: 0,
            name: "".parse().unwrap(),
            element: Element::None,
            class: Rank::G4,
            rank_item_on: 0,
            rank_item_off: 0,
            best_rank_item_on: 0,
            best_rank_item_off: 0,
            x_f4: 0,
            debug: false,
        }
    }
}

// 13
#[derive(Debug, Clone, DekuRead, DekuWrite)]
pub struct LobbyData {
    pub num: LobbyNum,
    pub member_max: i16,
    pub member: i16,
    pub name: WString<17>,
    pub unk: [u8; 32],
    pub mode: Mode,
}

#[derive(Debug, Clone, DekuRead, DekuWrite)]
pub struct RoomStat {
    pub room: RoomNum,
    pub flag: i8,
    pub member_max: i8,
    pub member: i8,
    pub watcher: i8,
    pub rules: i8,
    pub time_limit: i8,
    pub course: i8,
    pub season: i8,
    pub num_holes: i8,
    pub course_setting: i8,
    // need to review these for competition rooms
    // TODO: these bits are probably not mapped properly by deku
    #[deku(bits = 4)]
    pub limit_0: u8,
    #[deku(bits = 4)]
    pub limit_1: u8,
    #[deku(bits = 4)]
    pub limit_2: u8,
    #[deku(bits = 4)]
    pub limit_3: u8,
    #[deku(bits = 4)]
    pub limit_4: u8,
    #[deku(bits = 4)]
    pub limit_5: u8,
    #[deku(bits = 4)]
    pub limit_6: u8,
    #[deku(bits = 4)]
    pub limit_7: u8,
    #[deku(bits = 1)]
    pub limit_b_0: u8,
    #[deku(bits = 7)]
    pub limit_b_1: u8,
    #[deku(bits = 4)]
    pub limit_b_2: u8,
    #[deku(bits = 1)]
    pub limit_b_3: u8,
    #[deku(bits = 7, pad_bits_after = "12")]
    pub limit_b_4: u8,
}

// 19
#[derive(Debug, Clone, DekuRead, DekuWrite)]
pub struct Packet19 {
    pub mode: Mode,
    pub lobby: LobbyNum,
    pub room_stat: RoomStat,
    pub room_name: WString<33>,
    pub room_password: WString<17>,
}

impl Packet19 {
    pub fn create_error(mode: Mode, lobby: LobbyNum, code: i8) -> Packet19 {
        Packet19 {
            mode,
            lobby,
            room_stat: RoomStat {
                room: code,
                flag: 0,
                member_max: 0,
                member: 0,
                watcher: 0,
                rules: 0,
                time_limit: 0,
                course: 0,
                season: 0,
                num_holes: 0,
                course_setting: 0,
                limit_0: 0,
                limit_1: 0,
                limit_2: 0,
                limit_3: 0,
                limit_4: 0,
                limit_5: 0,
                limit_6: 0,
                limit_7: 0,
                limit_b_0: 0,
                limit_b_1: 0,
                limit_b_2: 0,
                limit_b_3: 0,
                limit_b_4: 0,
            },
            room_name: WString::default(),
            room_password: WString::default(),
        }
    }
}

#[derive(Debug, Clone, DekuRead, DekuWrite)]
pub struct UList {
    pub cid: CID,
    pub uid: UID,
    pub stat: u16,
    // TODO: double-check that this field is mapped as expected
    #[deku(bits = 1, pad_bits_before = "7", pad_bits_after = "24")]
    pub team: u8,
    pub mode: Mode,
    pub lobby: LobbyNum,
    pub room: RoomNum,
    pub pclass: Class,
    pub element: Element,
    pub title: u8,
    pub sv_no: i8,
    pub circle: i32,
    pub name: WString<19>,
}

#[derive(Debug, Clone, DekuRead, DekuWrite)]
pub struct UListL {
    pub cid: CID,
    pub uid: UID,
    pub stat: u16,
    // TODO: double-check that this field is mapped as expected
    #[deku(bits = 1, pad_bits_before = "7", pad_bits_after = "24")]
    pub team: u8, // assumed, this might be fake
    pub mode: Mode,
    pub lobby: LobbyNum,
    pub room: RoomNum,
    pub pclass: Class,    // or rank
    pub element: Element, // or att
    pub title: u8,
    pub circle: i32,
    pub name: WString<17>,
}

// 105
#[derive(Debug, Clone, DekuRead, DekuWrite)]
pub struct SetPlayerName {
    pub unk1: i32,
    pub unk2: i32,
    pub name: WString<17>,
    pub unk3: i32,
}

// 268
#[derive(Debug, Clone, DekuRead, DekuWrite)]
pub struct ModeCtrl {
    #[deku(bits = 1)]
    pub flags: [bool; 92],
}

// 309
#[derive(Debug, Clone, DekuRead, DekuWrite)]
pub struct SVItemData {
    pub item_code: u32,
    pub unk_0: [u8; 11],
    pub brand_index: i8,
    pub power: u8,
    pub control: u8,
    pub impact: u8,
    pub spin: u8,
    pub luck: u8,
    pub mood: u8,
    pub other_vars: [u8; 14],
    pub flags: u32,
}

// 312
#[derive(Debug, Clone, DekuRead, DekuWrite)]
pub struct ClubData {
    pub id: i16,
    pub power: f32,
    pub control: f32,
    pub impact: f32,
    pub spin: f32,
    pub luck: f32,
    #[deku(pad_bytes_after = "1")]
    pub x16: u8,
    pub distance: f32,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, DekuRead, DekuWrite)]
#[deku(type = "u8", bits = "3")]
pub enum Outcome {
    Invalid = 0,
    Aborted = 1,
    Lose = 2,
    Draw = 3,
    Win = 4,
    UnearnedWin = 5,
    Conv = 6,
}

impl Outcome {
    pub fn from_u32(val: u32) -> Self {
        match val {
            1 => Self::Aborted,
            2 => Self::Lose,
            3 => Self::Draw,
            4 => Self::Win,
            5 => Self::UnearnedWin,
            6 => Self::Conv,
            _ => Self::Invalid,
        }
    }

    pub fn to_u32(self) -> u32 {
        match self {
            Self::Invalid => 0,
            Self::Aborted => 1,
            Self::Lose => 2,
            Self::Draw => 3,
            Self::Win => 4,
            Self::UnearnedWin => 5,
            Self::Conv => 6,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, DekuRead, DekuWrite)]
#[deku(type = "u8", bits = "3")]
pub enum QuickMatchHoleSetting {
    Hole3 = 0,
    Hole6 = 1,
    Hole9 = 2,
    Hole18 = 3,
    Random = 4,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, DekuRead, DekuWrite)]
#[deku(type = "u8", bits = "3")]
pub enum QuickMatchTimeSetting {
    Time30 = 0,
    Time60 = 1,
    Time90 = 2,
    Time120 = 3,
    Random = 4,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, DekuRead, DekuWrite)]
#[deku(type = "u8", bits = "2")]
pub enum QuickMatchRuleSetting {
    Stroke = 0,
    Match = 1,
    Random = 2,
}

#[derive(Debug, Clone, DekuRead, DekuWrite)]
pub struct Packet50Data {
    pub uid: UID,
    pub score: i16,
    pub x6: u8,
    pub win_combo: i8,

    // TODO: these bits are almost certainly not mapped properly by Deku
    // all fields from here up to (and including) time_setting are part of one u32
    #[deku(bits = 5)]
    pub server_id: i8,
    #[deku(bits = 5)]
    pub rank: i8,
    #[deku(bits = 5)]
    pub best_rank: i8,
    #[deku(bits = 1)]
    pub item_on: bool,
    // 2 bits
    pub rule_setting: QuickMatchRuleSetting,
    // 3 bits
    pub hole_setting: QuickMatchHoleSetting,
    // 3 bits
    #[deku(pad_bits_after = "8")]
    pub time_setting: QuickMatchTimeSetting,

    // unsure
    #[deku(bits = 2, pad_bits_after = "30")]
    pub unk: i8,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, DekuRead, DekuWrite)]
#[deku(type = "i8")]
pub enum BuyItemResult {
    OK = 0,
    Balance = -1,
    NoItem = -2,
    InvalidCount = -3,
    InvalidItemType = -4,
    Err = -5,
    NoTicket = -6,
}

#[derive(Debug, Clone, DekuRead, DekuWrite)]
pub struct ChrData {
    pub chr_uid: ChrUID,
    pub type_: i16, // this is a character ID!
    pub class: Class,
    pub x_7: i8,
    pub param_power: i16,
    pub param_control: i16,
    pub param_impact: i16,
    pub param_spin: i16,
    pub x_10: [u8; 16],
    pub param_settings: [ParamTuple; 8],
    pub appearance: Appearance,
    pub club: Item,
    pub ball: Item,
    pub caddie: Item,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, DekuRead, DekuWrite)]
#[deku(type = "i8")]
pub enum MailSendResult {
    OK = 0,
    UnknownError = -1,
    MailsDisabled = -2,
    LimitReached = -3,
    Err = -4,
}

#[derive(Debug, Clone, DekuRead, DekuWrite)]
pub struct Delivery {
    pub unk1: i32,
    pub dest_uid: UID,
    pub item: Item, // is this CountedItem?
    pub unk2: i32,
    pub delivery_index: i8,
    pub unk3: [i8; 3],
    pub msg: AString<361>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, DekuRead, DekuWrite)]
#[deku(type = "i8")]
pub enum SendDeliverResult {
    OK = 0,
    GenericError1 = -1,
    GenericError2 = -2,
    // The slot is no longer available because the item has been sent.
    SlotUnavailable = -3,
    GenericError4 = -4,
    // The item could not be delivered because the delivery box at the destination was full.
    LimitReached = -5,
    // I received incorrect item information.
    IncorrectItemInfo = -6,
    GenericError7 = -7,
    GenericError8 = -8,
    // Items cannot be sent to users registered as BLACK.
    UserIsBlocked = -9,
    // The item cannot be sent because the specified user has set the delivery refusal.
    DeliveryDisabled = -10,
    // We were unable to send the item due to insufficient tickets.
    NotEnoughTickets = -11,
    // I didn't receive the item because the sender canceled the delivery.
    SenderCancelledDelivery = -12,
    // The item could not be sent because the sender canceled the trade.
    SenderCancelledTrade = -13,
}

#[derive(Debug, Clone, DekuRead, DekuWrite)]
pub struct CaddieData {
    pub unk1: i32,
    pub id: i32,
    pub datetime: DateTime,
    pub unk2: i32,
}

#[derive(Debug, Clone, DekuRead, DekuWrite)]
pub struct Packet183Entry {
    pub uid: UID,
    pub value: i32,
    pub unk: i32,
    pub status_icon: i8,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, DekuRead, DekuWrite)]
#[deku(type = "i8")]
pub enum ChgUDataType {
    // 'uid' has accepted the outbound friend request coming from 'p0'
    FriendRequestAccepted = 0,
    // 'uid' has removed 'p0' as a friend
    RemoveFriend = 1,
    // 'uid' has sent a friend request to 'p0'
    FriendRequestReceived = 2,
    // 'uid' has revoked their friend request to 'p0',
    FriendRequestRevoked = 3,
    // 'uid' has rejected the outbound friend request coming from 'p0'
    FriendRequestRejected = 4,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, DekuRead, DekuWrite, Serialize, Deserialize)]
#[deku(type = "i8")]
pub enum Element {
    None = -1,
    Blue = 0,
    Red = 1,
    Green = 2,
    Yellow = 3,
    Pink = 4,
}
