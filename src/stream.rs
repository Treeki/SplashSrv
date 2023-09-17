use anyhow::Result;
use bytes::{Buf, BytesMut};
use deku::{DekuContainerRead, DekuContainerWrite, DekuEnumExt};
use log::{debug, error};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use tokio_rustls::server::TlsStream;

use crate::packets::{EntirePacket, Packet, PacketHeader};

pub struct Connection {
    stream: TlsStream<TcpStream>,
    buffer: BytesMut,
    next_pid: i16,
}

impl Connection {
    pub fn new(stream: TlsStream<TcpStream>) -> Connection {
        Connection {
            stream,
            buffer: BytesMut::with_capacity(4 * 1024),
            next_pid: 1,
        }
    }

    pub async fn read_packet(&mut self) -> Result<Option<EntirePacket>> {
        loop {
            if let Some(packet) = self.parse_packet()? {
                return Ok(Some(packet));
            }

            // try and read more data
            if 0 == self.stream.read_buf(&mut self.buffer).await? {
                return Ok(None);
            }
        }
    }

    pub async fn write_packet(&mut self, packet: Packet) -> Result<()> {
        let pid = self.next_pid;
        self.next_pid += 1;
        self.write_packet_with_pid(packet, pid).await
    }

    pub async fn write_packet_with_pid(&mut self, packet: Packet, pid: i16) -> Result<()> {
        let id = packet.deku_id()?;

        let packet = EntirePacket {
            header: PacketHeader { id, pid },
            packet,
        };
        let data = packet.to_bytes()?;

        debug!("writing {data:?}");
        self.stream.write_u16_le(data.len().try_into()?).await?;
        self.stream.write_all(&data).await?;
        Ok(())
    }

    fn parse_packet(&mut self) -> Result<Option<EntirePacket>> {
        // can we grab the packet size?
        if self.buffer.len() < 2 {
            return Ok(None);
        }

        let packet_size: [u8; 2] = self.buffer[..2].try_into()?;
        let packet_size: usize = u16::from_le_bytes(packet_size).into();
        if self.buffer.len() < (packet_size + 2) {
            return Ok(None);
        }

        // we should have enough data
        let payload = &self.buffer[2..2 + packet_size];
        let (_remain, packet) = match EntirePacket::from_bytes((payload, 0)) {
            Ok(p) => p,
            Err(e) => {
                let mut buf = String::new();
                for b in payload {
                    buf.push_str(&format!("{b:02x}"));
                }
                error!("failed to parse packet: [{buf}]");
                return Err(e.into());
            }
        };

        self.buffer.advance(2 + packet_size);

        Ok(Some(packet))
    }

    pub async fn shutdown(&mut self) -> Result<()> {
        self.stream.shutdown().await?;
        Ok(())
    }
}
