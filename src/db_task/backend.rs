use anyhow::{bail, Result};
use log::error;
use rusqlite::{params, Connection, OptionalExtension};
use rusqlite_migration::{Migrations, M};

use crate::{
    data::{record::CRecord, Account, Appearance, Character, User},
    packets::{ChrUID, UID},
};

use super::Command;

pub(super) struct DB {
    conn: Connection,
}

impl DB {
    fn authenticate_user(&mut self, login_id: String) -> Result<Option<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT password FROM accounts WHERE login_id = ?1")?;
        let password_hash = stmt
            .query_row(params![login_id], |row| Ok(row.get(0)?))
            .optional()?;
        Ok(password_hash)
    }

    fn authenticate_user_to_game(&mut self, login_id: String, password: String) -> Result<Account> {
        let mut stmt = self
            .conn
            .prepare("SELECT uid, password, name, data FROM accounts WHERE login_id = ?1")?;
        let (uid, password_hash, name, data): (UID, String, Option<String>, Option<String>) = stmt
            .query_row([login_id], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })?;

        // TODO: use actual hashing here
        if password != password_hash {
            bail!("bad password at game server")
        }

        let user = match data {
            Some(data) => serde_json::from_str(&data)?,
            // New accounts will have no data here
            None => Default::default(),
        };

        // fetch all of this user's characters
        let mut characters = Vec::new();
        let mut stmt = self
            .conn
            .prepare("SELECT chr_uid, data FROM characters WHERE uid = ?1")?;
        for row in stmt.query_map([uid], |row| Ok((row.get(0)?, row.get(1)?)))? {
            let (chr_uid, data): (ChrUID, String) = row?;
            let character = serde_json::from_str(&data)?;
            characters.push((chr_uid, character));
        }

        Ok(Account {
            uid,
            name,
            user,
            characters,
        })
    }

    fn write_user(&mut self, uid: UID, data: User) -> Result<()> {
        let mut stmt = self
            .conn
            .prepare("UPDATE accounts SET data = ?1 WHERE uid = ?2")?;
        let data = serde_json::to_string(&data)?;
        stmt.execute(params![data, uid])?;
        Ok(())
    }

    fn set_player_name(&mut self, uid: UID, name: String) -> Result<()> {
        if name.is_empty() {
            bail!("name cannot be empty")
        }

        // is this name already in use?
        let mut stmt = self
            .conn
            .prepare("SELECT uid FROM accounts WHERE name = ?1")?;
        let existing: Option<UID> = stmt.query_row([&name], |row| row.get(0)).optional()?;
        if let Some(existing) = existing {
            if existing == uid {
                // this player already has this name, so this is a no-op
                return Ok(());
            } else {
                // this name is taken
                bail!("username in use")
            }
        }

        let mut stmt = self
            .conn
            .prepare("UPDATE accounts SET name = ?1 WHERE uid = ?2")?;
        stmt.execute(params![name, uid])?;
        Ok(())
    }

    fn create_character(
        &mut self,
        uid: UID,
        appearance: Appearance,
    ) -> Result<(ChrUID, Character)> {
        // first, ensure that this user does not already have a character
        let mut stmt = self
            .conn
            .prepare("SELECT COUNT(*) FROM characters WHERE uid = ?1")?;
        let count: usize = stmt.query_row([uid], |row| row.get(0))?;
        if count > 0 {
            bail!("character already exists");
        }

        // now add them
        let character = Character::new(appearance);

        let mut stmt = self
            .conn
            .prepare("INSERT INTO characters (uid, data) VALUES (?1, ?2)")?;
        let data = serde_json::to_string(&character)?;
        let chr_uid = stmt.insert(params![uid, data])?;
        let chr_uid: ChrUID = chr_uid.try_into()?;

        Ok((chr_uid, character))
    }

    fn write_character(&mut self, chr_uid: ChrUID, data: Character) -> Result<()> {
        let mut stmt = self
            .conn
            .prepare("UPDATE characters SET data = ?1 WHERE chr_uid = ?2")?;
        let data = serde_json::to_string(&data)?;
        stmt.execute(params![data, chr_uid])?;
        Ok(())
    }

    fn get_c_record(&mut self, uid: UID, course: i8, season: i8, holes: i8) -> Result<CRecord> {
        let mut stmt = self
            .conn
            .prepare("SELECT data FROM c_records WHERE uid = ?1 AND key = ?2")?;
        let key = ((course as i32) * 32) + ((season as i32) * 4) + (holes as i32);

        let data: Option<String> = stmt
            .query_row(params![uid, key], |row| row.get(0))
            .optional()?;
        if let Some(data) = data {
            let data: CRecord = serde_json::from_str(&data)?;
            Ok(data)
        } else {
            // doesn't already exist, so whatever
            Ok(CRecord::default())
        }
    }

    pub(super) fn handle_command(&mut self, command: Command) -> bool {
        match command {
            Command::AuthenticateUser { login_id, resp } => {
                resp.send(self.authenticate_user(login_id)).is_ok()
            }
            Command::AuthenticateUserToGame {
                login_id,
                password,
                resp,
            } => resp
                .send(self.authenticate_user_to_game(login_id, password))
                .is_ok(),
            Command::WriteUser { uid, data } => match self.write_user(uid, data) {
                Ok(()) => true,
                Err(e) => {
                    error!("failed to save user {uid}: {e:?}");
                    false
                }
            },
            Command::SetPlayerName { uid, name, resp } => {
                resp.send(self.set_player_name(uid, name)).is_ok()
            }
            Command::CreateCharacter {
                uid,
                appearance,
                resp,
            } => resp.send(self.create_character(uid, appearance)).is_ok(),
            Command::WriteCharacter { chr_uid, data } => {
                match self.write_character(chr_uid, data) {
                    Ok(()) => true,
                    Err(e) => {
                        error!("failed to save character {chr_uid}: {e:?}");
                        false
                    }
                }
            }
            Command::GetCRecord {
                uid,
                course,
                season,
                holes,
                resp,
            } => resp
                .send(self.get_c_record(uid, course, season, holes))
                .is_ok(),
        }
    }
}

pub(super) fn create() -> Result<DB> {
    let migrations = Migrations::new(vec![
        M::up(
            "CREATE TABLE accounts(
				uid INTEGER PRIMARY KEY NOT NULL,
				login_id TEXT NOT NULL,
				name TEXT,
				password TEXT NOT NULL,
				data TEXT
			);",
        ),
        M::up(
            "CREATE TABLE characters(
				chr_uid INTEGER PRIMARY KEY NOT NULL,
				uid INTEGER NOT NULL,
				data TEXT,
				FOREIGN KEY (uid) REFERENCES accounts(uid)
			);",
        ),
        M::up(
            "CREATE TABLE c_records(
				uid INTEGER NOT NULL,
				key INTEGER NOT NULL,
				data TEXT
			);",
        ),
    ]);

    let mut conn = Connection::open("splashsrv.db")?;

    migrations.to_latest(&mut conn)?;

    let db = DB { conn };
    Ok(db)
}
