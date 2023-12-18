#![allow(unused)]

use std::io::prelude::*;
use std::net::TcpStream;
use std::net::TcpListener;
use std::io::BufReader;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    println!("Connection established!");

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();

        loop {
            println!("listening...");
            let buf_reader = BufReader::new(&mut stream);
            let request_data: Vec<_> = buf_reader
                .lines()
                .map(|result| result.unwrap())
                .take_while(|line| !line.is_empty())
                .collect();

            let request_msg = request_data.join("\n") + "\r\n\r\n";
            println!("received: {}", request_msg);

            stream.write_all(request_msg.as_bytes()).unwrap();
            println!("responded with: {:?}", &request_msg);
        }
    }
}
