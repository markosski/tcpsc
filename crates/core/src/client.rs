use std::time::Duration;
use std::io::BufReader;
use std::io::Write;
use std::net::{SocketAddr, TcpStream};
use log::{debug, error};
use crate::models::{Header, Response};
use crate::models::Message;
use crate::utils::{GeneralError,Result};

pub struct Client {
    tcp: TcpStream
}

impl Client {
    pub fn new(ip: [u8; 4], port: u16, timeout_sec: u64) -> Result<Client> {
        match TcpStream::connect(SocketAddr::from((ip, port))) {
            Ok(tcp) => {
                let _ = tcp.set_read_timeout(Some(Duration::new(timeout_sec, 0)));
                Ok(Client {tcp})
            },
            Err(e) => Err(GeneralError::new(format!("{}", e)))
        }
    }

    pub fn send(&mut self, header: Header, data: Vec<u8>) -> Result<Response> {
        let message = Message::new(header, data);

        match self.tcp.write_all(&message.to_bytes()[..]) {
            Ok(_) => debug!("data sent: {:?}", &message.to_bytes()),
            Err(err) => error!("error responding to server; {}", err.to_string())
        };

        let mut buf_reader = BufReader::new(&mut self.tcp);

        Response::from_buffer(&mut buf_reader)
    }

    pub fn close(&self) {
        self.tcp.shutdown(std::net::Shutdown::Both).unwrap();
    }
}