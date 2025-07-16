use openssl::ssl::{SslConnector, SslMethod};
use std::{io::{Read, Write}, net::TcpStream};

mod core;
mod utils;
mod control;
mod file;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = SslConnector::builder(SslMethod::tls())?;
    builder.set_ca_file("cert.pem")?; 
    let connector = builder.build();

    let stream = TcpStream::connect("127.0.0.1:17878")?;
    println!("Connected to server");

    let mut ssl_stream = connector.connect("localhost", stream)?;
    println!("SSL handshake successful");

    ssl_stream.write_all(b"ping")?;
    let mut buffer = [0; 1024];
    let n = ssl_stream.read(&mut buffer)?;
    println!("Received: {}", String::from_utf8_lossy(&buffer[0..n]));

    Ok(())
}