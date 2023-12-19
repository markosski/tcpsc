#![allow(unused)]

use std::io::prelude::*;
use std::io::BufReader;
use threadpool::ThreadPool;
use std::thread;
use std::net::{SocketAddr, TcpListener, TcpStream};

fn main() {
    let port = 7878;
    let listener = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], port)),).unwrap();
    let pool = ThreadPool::new(4);

    println!("Server started on port {}!", &port);
    loop {
        match listener.accept() {
            Ok((stream, addr)) => {
                println!("incoming connection from remote client: {}:{}", &addr.ip(), &addr.port());
                pool.execute(|| {
                    println!("starting session on thread {:?}...", &thread::current().id());
                    handler(stream);
                    println!("thread {:?} completed", &thread::current().id());
                });
            },
            Err(e) => println!("couldn't get client: {e:?}"),
        }
    }
}

fn handler(mut stream: TcpStream) {
    loop {
        let buf_reader = BufReader::new(&mut stream);
        let request_data: Vec<_> = buf_reader
            .lines()
            .map(|result| result.unwrap())
            .take_while(|line| !line.is_empty())
            .collect();

        println!("received: {:?}", &request_data);

        if request_data.len() == 0 {
            println!("ending session...");
            break;
        }

        let response_msg = request_data.join("");

        let response_data = response_msg + "\n\n"; // extra blank line is needed to indicate EOF
        match stream.write_all(response_data.as_bytes()) {
            Ok(ok) => println!("response sent successfully!"),
            Err(err) => println!("error sending response; {}", err.to_string())
        }
    }
}
