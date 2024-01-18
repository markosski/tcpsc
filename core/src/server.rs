#![allow(unused)]

use std::io::Error;
use std::io::prelude::*;
use std::io::BufReader;
use std::time::Duration;
use threadpool::ThreadPool;
use std::thread;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::env;
use std::str;
use std::sync::Arc;
use std::io::ErrorKind;
use std::sync::mpsc;
use crate::utils;
use crate::models::{Header, Message, Response};

pub struct Server<'a> {
    header_size: usize,
    thread_pool: &'a ThreadPool
}

struct HandlerMessage {
    stream: TcpStream,

}

impl<'a> Server<'a> {
    pub fn new(header_size: usize, thread_pool: &'a ThreadPool) -> Server {
        Server { 
            header_size: header_size,
            thread_pool: thread_pool
        }
    }

    pub fn serve<F>(&self, port: u16, handler: Arc<F>) -> Vec<u8> 
    where 
        F: Fn(&Vec<u8>, &Vec<u8>) -> Vec<u8> + Send + Sync + 'static
    {
        let listener = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], port)),).unwrap();

        println!("server started on port: {} with connection pool size: {}!", &port, &self.thread_pool.max_count());
        loop {
            match listener.accept() {
                Ok((stream, addr)) => {
                    println!("incoming connection from remote client: {}:{}", &addr.ip(), &addr.port());
                    if self.thread_pool.active_count() == self.thread_pool.max_count() {
                        println!("connection pool full")
                    } else {
                        let handler2 = handler.clone();
                        let header_size = self.header_size.clone();

                        self.thread_pool.execute(move || {
                            println!("starting session on thread {:?}...", &thread::current().id());

                            loop {
                                let t = thread::spawn(|| {
                                    let response = Server::handle_message(stream, header_size, handler2);
                                });
                                thread::sleep(std::time::Duration::new(2, 0));
                            }

                            println!("session completed on thread {:?}", &thread::current().id());
                        });
                    }
                },
                Err(e) => println!("couldn't get client: {e:?}"),
            }
        } 
    }

    fn handle_message<F>(mut stream: TcpStream, header_size: usize, handler: Arc<F>) 
    where 
        F: Fn(&Vec<u8>, &Vec<u8>) -> Vec<u8> + Send + Sync
    {
        let header_size_less_one_byte = header_size - 1;
        loop {
            println!("top of the loop...");

            let mut buf_reader = BufReader::new(&mut stream);
    
            let mut format_buf: Vec<u8> = vec![];
            buf_reader.get_mut().take(Header::size() as u64).read_to_end(&mut format_buf);
            let header = Header::from_bytes(&format_buf).expect("malformed header");
    
            let header_string = str::from_utf8(&format_buf[..]).unwrap();
    
            if header.message_length == 0 {
                println!("message size is empty...");
                continue;
            } else {
                println!("message size is: {}", &header.message_length);
            }
    
            println!("reading data...");
            let mut data: Vec<u8> = Vec::with_capacity(header.message_length as usize);
            buf_reader.read_exact(&mut data[..]);
            buf_reader.take(header.message_length as u64).read_to_end(&mut data);
    
            println!("received: {:?}", str::from_utf8(&data));
    
            let mut data = handler(&format_buf, &data);
            let data_size = data.len() as u32;
            let response = Response::new(data_size, data);
    
            match stream.write_all(&response.to_bytes()[..]) {
                Ok(ok) => println!("response sent successfully!"),
                Err(err) => println!("error sending response; {}", err.to_string())
            }
        }
    }    
}