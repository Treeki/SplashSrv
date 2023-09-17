use anyhow::Result;
use tokio::sync::{mpsc, oneshot};

use crate::{
    data::{record::CRecord, Account, Appearance, Character, User},
    packets::{ChrUID, UID},
};

use super::Command;

#[derive(Clone)]
pub struct DBTask {
    pub(super) tx: mpsc::Sender<Command>,
}

impl DBTask {
    pub async fn authenticate_user(&self, login_id: String) -> Result<Option<String>> {
        let (resp, rx) = oneshot::channel();
        self.tx
            .send(Command::AuthenticateUser { login_id, resp })
            .await
            .unwrap();
        rx.await?
    }

    pub async fn authenticate_user_to_game(
        &self,
        login_id: String,
        password: String,
    ) -> Result<Account> {
        let (resp, rx) = oneshot::channel();
        self.tx
            .send(Command::AuthenticateUserToGame {
                login_id,
                password,
                resp,
            })
            .await
            .unwrap();
        rx.await?
    }

    pub async fn write_user(&self, uid: UID, data: User) {
        self.tx
            .send(Command::WriteUser { uid, data })
            .await
            .unwrap();
    }

    pub async fn set_player_name(&self, uid: UID, name: String) -> Result<()> {
        let (resp, rx) = oneshot::channel();
        self.tx
            .send(Command::SetPlayerName { uid, name, resp })
            .await
            .unwrap();
        rx.await?
    }

    pub async fn create_character(
        &self,
        uid: UID,
        appearance: Appearance,
    ) -> Result<(ChrUID, Character)> {
        let (resp, rx) = oneshot::channel();
        self.tx
            .send(Command::CreateCharacter {
                uid,
                appearance,
                resp,
            })
            .await
            .unwrap();
        rx.await?
    }

    pub async fn write_character(&self, chr_uid: ChrUID, data: Character) {
        self.tx
            .send(Command::WriteCharacter { chr_uid, data })
            .await
            .unwrap();
    }

    pub async fn get_c_record(
        &self,
        uid: UID,
        course: i8,
        season: i8,
        holes: i8,
    ) -> Result<CRecord> {
        let (resp, rx) = oneshot::channel();
        self.tx
            .send(Command::GetCRecord {
                uid,
                course,
                season,
                holes,
                resp,
            })
            .await
            .unwrap();
        rx.await?
    }
}
