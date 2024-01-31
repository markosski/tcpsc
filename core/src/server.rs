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
use std::sync::{Arc,Mutex,MutexGuard};
use std::io::ErrorKind;
use std::io::Read;
use std::sync::mpsc;
use crate::utils;
use crate::models::{Header, Message, Response};

pub struct Server<'a> {
    header_size: usize,
    timeout_seconds: usize,
    thread_pool: &'a ThreadPool
}

struct HandlerMessage {
    stream: TcpStream,

}

impl<'a> Server<'a> {
    pub fn new(header_size: usize, timeout_seconds: usize, thread_pool: &'a ThreadPool) -> Server {
        Server { 
            header_size: header_size,
            timeout_seconds: timeout_seconds,
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
                Ok((mut stream, addr)) => {
                    println!("incoming connection from remote client: {}:{}", &addr.ip(), &addr.port());

                    if self.thread_pool.active_count() == self.thread_pool.max_count() {
                        println!("can't accept any more connections")
                    } else {
                        let handler_ref = handler.clone();
                        let header_size = self.header_size.clone();
                        let timeout_seconds = self.timeout_seconds.clone();

                        self.thread_pool.execute(move || {
                            println!("starting session on thread {:?}...", &thread::current().id());

                            loop {
                                let mut tcp = match stream.try_clone() {
                                    Ok(ok) => ok,
                                    Err(err) => {
                                        println!("error");
                                        break;
                                    }
                                };

                                let mut format_buf: Vec<u8> = vec![];

                                let response = Server::handle_message(&tcp, header_size, timeout_seconds, handler_ref.clone());
                                match response {
                                    Ok(resp) => {
                                        match tcp.write_all(&resp.to_bytes()[..]) {
                                            Ok(ok) => println!("response sent successfully!"),
                                            Err(err) => println!("error sending response; {}", err.to_string())
                                        }
                                    }
                                    Err(err) => {
                                        println!("handler did not complete successfully or client disconnected");
                                        break;
                                    }
                                }
                            }

                            println!("session completed on thread {:?}", &thread::current().id());
                        });
                    }
                },
                Err(e) => println!("couldn't get client: {e:?}"),
            }
        } 
    }

    fn handle_message<F>(mut tcp: &TcpStream, header_size: usize, timeout: usize, handler: Arc<F>) -> Result<Response, Error>
    where 
        F: Fn(&Vec<u8>, &Vec<u8>) -> Vec<u8> + Send + Sync
    {
        let header_size_less_one_byte = header_size - 1;

        println!("awaiting data...");
        tcp.set_read_timeout(Some(Duration::from_secs(timeout as u64)));

        let mut format_buf: Vec<u8> = vec![];
        let mut peek_buf: Vec<u8> = vec![];
        tcp.take(1).read_to_end(&mut format_buf);
        if format_buf.len() > 0 {
            println!("detected data");
            tcp.set_read_timeout(Some(Duration::from_secs(5)));
        }
        tcp.take((Header::size() - 1) as u64).read_to_end(&mut format_buf);
        let header = Header::from_bytes(&format_buf)?;
        println!("completed reading header...");

        let header_string = str::from_utf8(&format_buf[..]).unwrap();

        if header.message_length == 0 {
            println!("message size is empty...");
            return Err(Error::new(ErrorKind::Other, "message size is empty"));
        } else {
            println!("message size is: {}", &header.message_length);
        }

        println!("reading data...");
        let mut data: Vec<u8> = Vec::with_capacity(header.message_length as usize);
        tcp.read_exact(&mut data[..]);
        tcp.take(header.message_length as u64).read_to_end(&mut data);

        println!("received: {:?}", str::from_utf8(&data));

        let mut data = handler(&format_buf, &data);
        let data_size = data.len() as u32;
        Ok(Response::new(data_size, data))
    }    
}