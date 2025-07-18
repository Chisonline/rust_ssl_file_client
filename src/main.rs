use crate::core::client::ClientConfig;

mod core;
mod utils;
mod control;
mod file;
mod user;
mod terminal;

#[tokio::main]
async fn main() -> ! {

    let config = ClientConfig {
        cert_file: "cert.pem".to_string(),
        addr: "127.0.0.1".to_string(),
        port: 17878,
        domain: "localhost".to_string(),
        debug: false
    };

    core::client::init_config(config).await;

    terminal::terminal().await
}