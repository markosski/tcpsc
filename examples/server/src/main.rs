#![allow(unused)]

use std::io::prelude::*;
use std::io::BufReader;
use std::time::Duration;
use threadpool::ThreadPool;
use std::thread;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::io::{Error, ErrorKind};
use std::env;
use std::str;
use core::server::Server;
use core::models::Header;
extern crate env_logger;

fn handler(header: &Header, payload: &Vec<u8>) -> Result<Vec<u8>, Error> {
    let message_type = str::from_utf8(&header.message_type).unwrap();
    let payload_string = str::from_utf8(&payload).unwrap();
    println!("message string is {}", &message_type);

    match message_type {
        "REQ " => {
            println!("Command matched to {}", message_type);
            let payload_str = str::from_utf8(&payload).unwrap();
            Ok((String::from("response data: ") + payload_str).as_bytes().to_vec())
        }
        _ => Err(Error::new(ErrorKind::Other, format!("unrecognized command {}", &message_type)))
    }
}

fn main() {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Please provide port number");
    }
    static POOL_SIZE: usize = 2;
    let port = args[1].parse().expect("valid port number");
    let pool: ThreadPool = ThreadPool::new(POOL_SIZE);

    let server = Server::new(&pool);
    server.serve(port, handler);
}