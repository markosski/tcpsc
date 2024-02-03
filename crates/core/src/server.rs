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
use log::{debug, info, warn, error};
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
    pub fn new(thread_pool: &'a ThreadPool) -> Server {
        Server { 
            header_size: 16,
            timeout_seconds: 15,
            thread_pool: thread_pool
        }
    }

    pub fn set_header_size(&mut self, header_size: usize) {
        self.header_size = header_size;
    }

    pub fn set_timeout(&mut self, timeout_seconds: usize) {
        self.timeout_seconds = timeout_seconds;
    }

    pub fn serve<F>(&self, port: u16, handler: F) -> Vec<u8> 
    where 
        F: Fn(&Message) -> Result<Vec<u8>, Error> + Send + Sync + 'static
    {
        let handler_ref_arc = Arc::new(handler);
        let listener = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], port)),).unwrap();

        info!("server started on port: {} with connection pool size: {}!", &port, &self.thread_pool.max_count());
        loop {
            match listener.accept() {
                Ok((mut stream, addr)) => {
                    info!("incoming connection from remote client: {}:{}", &addr.ip(), &addr.port());

                    if self.thread_pool.active_count() == self.thread_pool.max_count() {
                        error!("can't accept any more connections")
                    } else {
                        let handler_ref = handler_ref_arc.clone();
                        let header_size = self.header_size.clone();
                        let timeout_seconds = self.timeout_seconds.clone();

                        self.thread_pool.execute(move || {
                            info!("starting session on thread {:?}...", &thread::current().id());

                            loop {
                                let mut tcp = match stream.try_clone() {
                                    Ok(ok) => ok,
                                    Err(err) => {
                                        error!("{}", err);
                                        break;
                                    }
                                };

                                let mut format_buf: Vec<u8> = vec![];

                                let response = Server::handle_message(&tcp, header_size, timeout_seconds, handler_ref.clone());
                                match response {
                                    Ok(resp) => {
                                        match tcp.write_all(&resp.to_bytes()[..]) {
                                            Ok(ok) => info!("response sent successfully!"),
                                            Err(err) => error!("error sending response; {}", err.to_string())
                                        }
                                    }
                                    Err(err) => {
                                        error!("request handler did not complete successfully or client disconnected");
                                        break;
                                    }
                                }
                            }

                            info!("session completed on thread {:?}", &thread::current().id());
                        });
                    }
                },
                Err(e) => error!("couldn't get client: {e:?}"),
            }
        } 
    }

    fn handle_message<F>(mut tcp: &TcpStream, header_size: usize, timeout: usize, handler: Arc<F>) -> Result<Response, Error>
    where 
        F: Fn(&Message) -> Result<Vec<u8>, Error> + Send + Sync
    {
        let header_size_less_one_byte = header_size - 1;

        info!("awaiting data...");
        tcp.set_read_timeout(Some(Duration::from_secs(timeout as u64)));

        let mut format_buf: Vec<u8> = vec![];
        let mut peek_buf: Vec<u8> = vec![];
        tcp.take(1).read_to_end(&mut format_buf);
        if format_buf.len() > 0 {
            debug!("detected data");
            tcp.set_read_timeout(Some(Duration::from_secs(5)));
        }
        tcp.take((Header::size() - 1) as u64).read_to_end(&mut format_buf);
        let header = Header::from_bytes(&format_buf)?;
        debug!("completed reading header...");

        let header_string = str::from_utf8(&format_buf[..]).unwrap();

        if header.message_length == 0 {
            debug!("message size is empty...");
            return Err(Error::new(ErrorKind::Other, "message size is empty"));
        } else {
            debug!("message size is: {}", &header.message_length);
        }

        debug!("reading data...");
        let mut data: Vec<u8> = Vec::with_capacity(header.message_length as usize);
        tcp.read_exact(&mut data[..]);
        tcp.take(header.message_length as u64).read_to_end(&mut data);

        debug!("received: {:?}", str::from_utf8(&data));
        let message = Message{header, data};

        match handler(&message) {
            Ok(resp) => Ok(Response::success(resp)),
            Err(e) => Ok(Response::error(format!("{}", e)))
        }
    }    
}