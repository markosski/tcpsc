#![allow(unused)]

use std::io::prelude::*;
use std::io::BufReader;
use std::time::Duration;
use std::thread;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::io::{Error, ErrorKind};
use std::env;
use std::str;
use core::server::Server;
use core::models::Message;
use core::utils::{Result,GeneralError};
extern crate env_logger;

fn handler(msg: &Message) -> Result<Vec<u8>> {
    let message_type = str::from_utf8(&msg.header.message_type)?;
    let payload_string = str::from_utf8(&msg.data)?;
    println!("message string is {}", &message_type);

    match message_type {
        "REQ " => {
            println!("Command matched to {}", message_type);
            let payload_str = str::from_utf8(&msg.data)?;
            Ok((String::from("response data: ") + payload_str).as_bytes().to_vec())
        }
        _ => Err(GeneralError::new( format!("unrecognized command {}", &message_type)))
    }
}

fn main() {
    env_logger::init();
    let rt = tokio::runtime::Runtime::new().unwrap();

    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Please provide port number");
    }
    let port = args[1].parse().expect("valid port number");

    let mut server = Server::new(2);
    server.set_timeout(60);
    
    rt.block_on(server.serve(port, handler));
}