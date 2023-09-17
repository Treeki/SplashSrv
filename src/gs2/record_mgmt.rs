use anyhow::Result;
use log::error;

use crate::{
    data::record::CRecord,
    packets::{Packet, Status, UID},
};

use super::GameServer;

impl GameServer {
    pub(super) async fn handle_get_c_record(
        &self,
        pid: i16,
        who: usize,
        uid: UID,
        course: i8,
        season: i8,
        hole_idx: i8,
    ) -> Result<()> {
        let packet = match self.db.get_c_record(uid, course, season, hole_idx).await {
            Ok(data) => Packet::SEND_CRECORD {
                uid,
                course,
                season,
                hole_idx,
                data,
                status: Status::OK,
            },
            Err(e) => {
                error!("error fetching CRecord: {e:?}");
                Packet::SEND_CRECORD {
                    uid,
                    course,
                    season,
                    hole_idx,
                    data: CRecord::default(),
                    status: Status::Err,
                }
            }
        };
        self.conns[who].write_with_pid(packet, pid).await?;

        Ok(())
    }
}
