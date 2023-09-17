use std::sync::Arc;

use anyhow::Result;
use log::{error, info, warn};
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio_rustls::rustls::ServerConfig;
use tokio_rustls::TlsAcceptor;

use crate::db_task::DBTask;
use crate::packets::{AckIDPassResult, GmsvData, Packet};
use crate::stream::Connection;

async fn authenticate_user(
    db: &DBTask,
    username: String,
    password: String,
    version: u16,
) -> AckIDPassResult {
    if username.is_empty() {
        return AckIDPassResult::IDError;
    }
    if password.is_empty() {
        return AckIDPassResult::PassError;
    }
    if version != 956 {
        return AckIDPassResult::VersionError;
    }

    let password_hash = match db.authenticate_user(username).await {
        Ok(Some(password_hash)) => password_hash,
        Ok(None) => return AckIDPassResult::AccountNotError,
        Err(e) => {
            error!("failed to auth user: {e:?}");
            return AckIDPassResult::AccountNotError;
        }
    };

    if password != password_hash {
        return AckIDPassResult::PassError;
    }

    AckIDPassResult::OK
}

async fn handle_connection(db: DBTask, tcp_stream: TcpStream, acceptor: TlsAcceptor) -> Result<()> {
    info!("Login connection from {}", tcp_stream.peer_addr()?);

    let tls_stream = acceptor.accept(tcp_stream).await?;
    let mut connection = Connection::new(tls_stream);
    let mut authenticated = false;

    while let Some(packet) = connection.read_packet().await? {
        match packet.packet {
            Packet::SEND_IDPASS(p) if !authenticated => {
                let username = p.username.to_string();
                let password = p.password.to_string();
                let version = p.version;
                let result = authenticate_user(&db, username, password, version).await;
                if result == AckIDPassResult::OK {
                    authenticated = true;
                }
                connection.write_packet(Packet::ACK_IDPASS(result)).await?;
            }

            Packet::REQ_GMSVLIST if authenticated => {
                let gmsv = GmsvData {
                    number: 1,
                    ip_address: "splash.wuffs.org".parse()?,
                    port: 2051,
                    enc_key: "i am not used".parse()?,
                    name: "CoolServer2".parse()?,
                    comment: "hewwo???".parse()?,
                    max: 20,
                    now: 1,
                };
                connection.write_packet(Packet::SEND_GMSVDATA(gmsv)).await?;
                connection.write_packet(Packet::ACK_GMSVLIST).await?;
            }
            _ => {
                warn!("[unhandled]");
            }
        }
    }

    info!("Login connection ending");
    connection.shutdown().await?;
    info!("Login connection ended");

    Ok(())
}

pub async fn run<A: ToSocketAddrs>(db: DBTask, config: Arc<ServerConfig>, addr: A) -> Result<()> {
    let acceptor = TlsAcceptor::from(config);
    let listener = TcpListener::bind(addr).await?;

    loop {
        let (stream, _) = listener.accept().await?;
        let acceptor = acceptor.clone();
        let db = db.clone();

        tokio::spawn(async move {
            let res = handle_connection(db, stream, acceptor).await;
            match res {
                Ok(_) => {}
                Err(err) => {
                    error!("{:?}", err);
                }
            }
        });
    }
}
