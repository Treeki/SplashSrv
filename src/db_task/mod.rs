use anyhow::Result;
use log::error;
use tokio::sync::{mpsc, oneshot};

mod backend;
mod frontend;

pub use frontend::DBTask;

use crate::{
    data::{record::CRecord, Account, Appearance, Character, User},
    packets::{ChrUID, UID},
};

enum Command {
    AuthenticateUser {
        login_id: String,
        resp: Responder<Result<Option<String>>>,
    },

    AuthenticateUserToGame {
        login_id: String,
        password: String,
        resp: Responder<Result<Account>>,
    },

    WriteUser {
        uid: UID,
        data: User,
    },

    SetPlayerName {
        uid: UID,
        name: String,
        resp: Responder<Result<()>>,
    },

    CreateCharacter {
        uid: UID,
        appearance: Appearance,
        resp: Responder<Result<(ChrUID, Character)>>,
    },

    WriteCharacter {
        chr_uid: ChrUID,
        data: Character,
    },

    GetCRecord {
        uid: UID,
        course: i8,
        season: i8,
        holes: i8,
        resp: Responder<Result<CRecord>>,
    },
}

type Responder<T> = oneshot::Sender<T>;

pub fn run() -> Result<DBTask> {
    let mut db = backend::create()?;
    let (tx, mut rx) = mpsc::channel(100);

    // TODO: should this be spawn_blocking?
    tokio::spawn(async move {
        while let Some(cmd) = rx.recv().await {
            if !db.handle_command(cmd) {
                error!("command failed");
            }
        }
    });

    Ok(DBTask { tx })
}
