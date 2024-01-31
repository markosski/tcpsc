
use std::io::{prelude::*, ErrorKind};
use std::time::Duration;
use std::io::BufReader;
use std::io::Error;
use std::io::Write;
use std::net::{SocketAddr, TcpStream};
use crate::models::Header;
use crate::models::Message;
use crate::utils;

pub struct Client {
    tcp: TcpStream
}

impl Client {
    pub fn new(ip: [u8; 4], port: u16, timeout_sec: u64) -> Result<Client, Error> {
        match TcpStream::connect(SocketAddr::from((ip, port))) {
            Ok(tcp) => {
                let _ = tcp.set_read_timeout(Some(Duration::new(timeout_sec, 0)));
                Ok(Client {tcp})
            },
            Err(e) => Err(Error::new(ErrorKind::Other, e.to_string()))
        }
    }

    pub fn send(&mut self, header: Header, data: Vec<u8>) -> Result<Vec<u8>, Error> {
        let message = Message::new(header, data);

        match self.tcp.write_all(&message.to_bytes()[..]) {
            Ok(_) => println!("data sent: {:?}", &message.to_bytes()),
            Err(err) => println!("error responding to server; {}", err.to_string())
        };

        let mut buf_reader = BufReader::new(&mut self.tcp);

        let mut message_length_bytes: [u8; 4] = [0; 4];
        buf_reader.read_exact(&mut message_length_bytes)?;

        let mut message: Vec<u8> = vec![];
        buf_reader.take(utils::as_u32_be(&message_length_bytes) as u64)
            .read_to_end(&mut message)?;

        Ok(message)
    }

    pub fn close(&self) {
        self.tcp.shutdown(std::net::Shutdown::Both).unwrap();
    }
}