#![allow(unused)]

use std::io::prelude::*;
use std::io::BufReader;
use std::time::Duration;
use threadpool::ThreadPool;
use std::thread;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::env;
use std::str;
use std::sync::Arc;
use core::server::Server;

fn handler(header: &Vec<u8>, payload: &Vec<u8>) -> Vec<u8> {
    let header_string = str::from_utf8(&header).unwrap();
    let payload_string = str::from_utf8(&payload).unwrap();
    println!("header string is {}", &header_string);

    let payload_str = str::from_utf8(&payload).unwrap();
    (String::from("response data: ") + payload_str).as_bytes().to_vec()
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Please provide port number");
    }
    static POOL_SIZE: usize = 2;
    let port = args[1].parse().expect("valid port number");
    let pool: ThreadPool = ThreadPool::new(POOL_SIZE);

    let server = Server::new(16, 15, &pool);
    server.serve(port, Arc::new(handler));
}