use anyhow::Result;
use log::{error, info, warn};
use tokio::{
    net::TcpStream,
    sync::{mpsc, oneshot},
    task::JoinHandle,
};
use tokio_rustls::TlsAcceptor;

use crate::{
    packets::{Packet, UData, CID},
    stream::Connection,
};

use super::{LoginResult, Message};

pub type ConnMessage = (Option<i16>, Packet);
pub type ConnSender = mpsc::Sender<ConnMessage>;
pub type ConnReceiver = mpsc::Receiver<ConnMessage>;

async fn do_handshake(
    gs2: mpsc::Sender<Message>,
    conn: &mut Connection,
) -> Result<Option<(CID, ConnReceiver)>> {
    while let Some(packet) = conn.read_packet().await? {
        if let Packet::SEND_IDPASS_G(p) = packet.packet {
            // Try to get ourselves in
            let (resp_tx, resp_rx) = oneshot::channel();
            gs2.send(Message::Login(p, resp_tx)).await?;

            match resp_rx.await? {
                LoginResult::Fail(code) => {
                    // No dice, just relay this to the client and keep trying.
                    let mut udata = UData::default();
                    udata.cid = code as i8 as CID;
                    conn.write_packet(Packet::ACK_IDPASS_G(udata)).await?;
                }

                LoginResult::Success { cid, packet_rx } => {
                    // We've established a session
                    // The server will send the initial ACK_IDPASS_G over the channel.
                    return Ok(Some((cid, packet_rx)));
                }
            }
        }
    }

    // Client disconnected without successfully handshaking.
    Ok(None)
}

async fn handle_connection(
    gs2: mpsc::Sender<Message>,
    stream: TcpStream,
    acceptor: TlsAcceptor,
) -> Result<()> {
    // Establish a TLS session
    let stream = acceptor.accept(stream).await?;
    let mut conn = Connection::new(stream);

    // Allow the client to log in
    let (cid, mut packet_rx) = match do_handshake(gs2.clone(), &mut conn).await? {
        Some(t) => t,
        None => return Ok(()),
    };

    // We are now authenticated with the server.
    // From this point on, we should not terminate without telling it beforehand.
    loop {
        tokio::select! {
            outbound_packet = packet_rx.recv() => {
                match outbound_packet {
                    None => {
                        // The server has kicked us off.
                        break;
                    }
                    Some((pid, packet)) => {
                        // This packet needs to go to the client
                        let result = match pid {
                            Some(pid) => conn.write_packet_with_pid(packet, pid).await,
                            None => conn.write_packet(packet).await
                        };

                        if let Err(e) = result {
                            // It's all over
                            warn!("Error writing to client: {e:?}");
                            gs2.send(Message::Logout(cid)).await?;
                            break;
                        }
                    }
                }
            }

            inbound_packet = conn.read_packet() => {
                match inbound_packet {
                    Ok(Some(packet)) => {
                        // This packet needs to go to the server
                        let pid = packet.header.pid;
                        let packet = packet.packet;
                        gs2.send(Message::PlayerData { cid, pid, packet }).await?;
                    }
                    Ok(None) => {
                        // Client disconnected
                        info!("Client disconnected!");
                        gs2.send(Message::Logout(cid)).await?;
                        break;
                    }
                    Err(e) => {
                        // Connection error
                        warn!("Error reading from client: {e:?}");
                        gs2.send(Message::Logout(cid)).await?;
                        break;
                    }
                }
            }
        }
    }

    conn.shutdown().await?;

    Ok(())
}

pub(super) fn run_connection(
    gs2: mpsc::Sender<Message>,
    stream: TcpStream,
    acceptor: TlsAcceptor,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        match handle_connection(gs2, stream, acceptor).await {
            Ok(_) => {}
            Err(err) => {
                error!("connection failed: {err:?}");
            }
        }
    })
}
