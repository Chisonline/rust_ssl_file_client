use anstyle::{Color, RgbColor, Style};
use base64::{engine::general_purpose, Engine as _};
use openssl::ssl::{SslConnector, SslMethod};
use serde::{Serialize, de::DeserializeOwned};
use std::{
    fmt::Debug, io::{Read as _, Write as _}, net::TcpStream
};

use crate::{
    control::ControlBlock,
    core::client::get_config, terminal::async_print,
};

#[derive(Debug)]
pub struct Payload<T>
where
    T: Serialize,
{
    pub method: String,
    pub block: Option<ControlBlock>,
    pub content: Option<T>,
}

async fn make_req<T>(payload: Payload<T>) -> String
where
    T: Serialize,
{
    let content = match payload.content {
        Some(content) => {
            let json_str = serde_json::to_string(&content).unwrap();
            general_purpose::STANDARD.encode(json_str)
        },
        None => "".to_string(),
    };
    let block = match payload.block {
        Some(block) => {
            let json_str = serde_json::to_string(&block).unwrap();
            general_purpose::STANDARD.encode(json_str)
        },
        None => ".".to_string(),
    };

    format!("{} {} {}", payload.method, block, content)
}

#[derive(Debug)]
pub struct Resp<R>
where
    R: DeserializeOwned,
{
    pub success: bool,
    pub block: Option<ControlBlock>,
    pub content: Option<R>,
}

pub async fn req_server<T, R>(payload: Payload<T>) -> Result<Resp<R>, Box<dyn std::error::Error>>
where T: Serialize + Debug, R:DeserializeOwned + Debug
{
    async_debug(format!("raw payload: {:?}", payload)).await;

    let req = make_req(payload).await;

    async_debug(format!("b64 payload: {}", req)).await;

    let resp = send_req(req).await?;

    async_debug(resp.clone()).await;

    let resp: Resp<R> = split_resp(resp).await;

    Ok(resp)
}

const END_MARK: &str = "\n\n\n";

async fn send_req(payload: String) -> Result<String, Box<dyn std::error::Error>> {
    let client_config = get_config().await;

    let addr = client_config.addr.clone();
    let port = client_config.port;
    let cert_file = client_config.cert_file.clone();
    let domain = &client_config.domain;

    let mut builder = SslConnector::builder(SslMethod::tls())?;
    builder.set_ca_file(cert_file)?;
    let connector = builder.build();

    let stream = TcpStream::connect(format!("{}:{}", addr, port))?;
    let mut ssl_stream = connector.connect(domain, stream)?;

    ssl_stream.write_all(format!("{}{}", payload, END_MARK).as_bytes())?;
    ssl_stream.flush()?;

    let mut buffer = Vec::new();
    let mut temp_buffer = [0; 1024];

    loop {
        let n = ssl_stream.read(&mut temp_buffer)?;
        if n == 0 {
            break;
        }
        buffer.extend_from_slice(&temp_buffer[0..n]);

        if buffer.ends_with(END_MARK.as_bytes()) {
            buffer.truncate(buffer.len() - END_MARK.len());
            break;
        }
    }

    let response = String::from_utf8(buffer)?;
    let response = response.trim();
    
    Ok(response.to_owned())
}

async fn split_resp<T>(resp: String) -> Resp<T>
where
    T: DeserializeOwned + Debug,
{
    let bytes = resp.bytes();
    async_debug(format!("{:?}", bytes)).await;

    let mut parts = resp.split(" ");

    let success = match parts.next().unwrap().to_string().as_str() {
        "true" => true,
        "false" => false,
        _ => false,
    };

    let block = match parts.next() {
        Some(str) => {
            match general_purpose::STANDARD.decode(str) {
                Ok(block_str) => serde_json::from_slice(&block_str).unwrap_or(None),
                Err(_) => None,
            }
        },
        None => None,
    };

    let content = match parts.next() {
        Some(str) => {
            match general_purpose::STANDARD.decode(str) {
                Ok(content_str) => serde_json::from_slice(&content_str).unwrap_or(None),
                Err(_) => None,
            }
        },
        None => None,
    };

    async_debug(format!("{} {:?} {:?}", success, block, content)).await;

    Resp {
        success,
        block,
        content,
    }
}

#[cfg(test)]
mod test {
    #[tokio::test]
    async fn test_split() {
        use super::*;
        let resp = "true . Mw==".to_string();
        let resp: Resp<u32> = split_resp(resp).await;
        assert_eq!(resp.success, true);
        assert_eq!(resp.content, Some(3));
        assert!(resp.block.is_none());
        println!("{:?}", resp)
    }
}

pub async fn async_debug(buffer: String) {

    let client_config = get_config().await;

    if !client_config.debug {
        return;
    }

    let style = Style::new().bold().fg_color(Some(Color::Rgb(RgbColor(128, 196, 0))));

    let buffer = format!("{style}[DEBUG]{style:#} {}", buffer);
    async_print(buffer).await
}