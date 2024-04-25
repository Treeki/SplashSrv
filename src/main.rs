use std::{fs::File, io::BufReader, sync::Arc};

use anyhow::Result;
use log::info;
use tokio_rustls::rustls::{Certificate, PrivateKey, ServerConfig};

mod data;
mod db_task;
mod gs2;
mod login_server;
mod packets;
mod stream;

fn load_config() -> Result<ServerConfig> {
    let mut reader = BufReader::new(File::open("cert.pem")?);
    let mut certs = Vec::new();
    let mut key = None;
    for item in rustls_pemfile::read_all(&mut reader)? {
        match item {
            rustls_pemfile::Item::X509Certificate(buf) => certs.push(Certificate(buf)),
            rustls_pemfile::Item::RSAKey(buf) => key = Some(PrivateKey(buf)),
            rustls_pemfile::Item::PKCS8Key(buf) => key = Some(PrivateKey(buf)),
            rustls_pemfile::Item::ECKey(buf) => key = Some(PrivateKey(buf)),
            _ => {}
        }
    }

    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs, key.expect("no key found"))?;

    Ok(config)
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let config = Arc::new(load_config()?);
    let db = db_task::run()?;
    let login_future = tokio::spawn(login_server::run(
        db.clone(),
        config.clone(),
        "0.0.0.0:2050",
    ));
    let game_future = tokio::spawn(gs2::run(db, config, "0.0.0.0:2051"));

    info!("starting server");
    let (login, game) = tokio::join!(login_future, game_future);
    login??;
    game??;
    Ok(())
}
