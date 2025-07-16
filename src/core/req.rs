use openssl::ssl::{SslConnector, SslMethod};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{
    io::{Read as _, Write as _},
    net::TcpStream,
};

use crate::{
    control::ControlBlock,
    core::{MAX_BLOCK_SIZE, client::get_config},
};

pub struct Payload<T>
where
    T: Serialize,
{
    method: String,
    block: Option<ControlBlock>,
    content: Option<T>,
}

async fn make_req<T>(payload: Payload<T>) -> String
where
    T: Serialize,
{
    let content = match payload.content {
        Some(content) => serde_json::to_string(&content).unwrap(),
        None => "".to_string(),
    };
    let block = match payload.block {
        Some(block) => serde_json::to_string(&block).unwrap(),
        None => ".".to_string(),
    };

    format!("{} {} {}", payload.method, block, content)
}

#[derive(Debug)]
pub struct Resp<R>
where
    R: DeserializeOwned,
{
    success: bool,
    block: Option<ControlBlock>,
    content: Option<R>,
}

pub async fn req<T, R>(payload: Payload<T>) -> Result<Resp<R>, Box<dyn std::error::Error>>
where T: Serialize, R:DeserializeOwned
{
    let req = make_req(payload).await;

    let resp = send_req(req).await?;

    let resp: Resp<R> = split_resp(resp).await;

    Ok(resp)
}

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

    ssl_stream.write_all(payload.as_bytes())?;

    let mut buffer = Vec::new();
    let mut temp_buffer = [0; 1024];

    loop {
        let n = ssl_stream.read(&mut temp_buffer)?;
        if n == 0 {
            break;
        }
        buffer.extend_from_slice(&temp_buffer[0..n]);
    }

    let response = String::from_utf8(buffer)?;

    Ok(response)
}

async fn split_resp<T>(resp: String) -> Resp<T>
where
    T: DeserializeOwned,
{
    let mut parts = resp.splitn(3, " ");

    let success = match parts.next().unwrap().to_string().as_str() {
        "true" => true,
        "false" => false,
        _ => false,
    };

    let block = match parts.next() {
        Some(str) => serde_json::from_str(str).unwrap_or(None),
        None => None,
    };

    let content = match parts.next() {
        Some(str) => serde_json::from_str(str).unwrap_or(None),
        None => None,
    };

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
        let resp = "true . 123".to_string();
        let resp: Resp<u32> = split_resp(resp).await;
        assert_eq!(resp.success, true);
        assert_eq!(resp.content, Some(123));
        assert!(resp.block.is_none());
        println!("{:?}", resp)
    }
}
