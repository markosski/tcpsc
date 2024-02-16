#![allow(unused)]

use crate::utils;
use crate::models::{Header, Message, Response};
use crate::utils::GeneralError;
use std::borrow::Borrow;
use std::io::Error;
use std::io::prelude::*;
use std::io::BufReader;
use std::time::Duration;
use std::thread;
use std::net::{SocketAddr};
use std::env;
use std::str;
use std::sync::{Arc,Mutex,MutexGuard};
use std::io::ErrorKind;
use std::io::Read;
use std::sync::mpsc;
use log::{debug, info, warn, error};
use utils::Result;
use tokio_io_timeout::TimeoutReader;
use tokio::io::AsyncWriteExt;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncRead;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::task::JoinHandle;
use tokio::pin;


pub struct Server {
    header_size: usize,
    timeout_seconds: usize,
    max_connection: usize
}

struct HandlerMessage {
    stream: TcpStream,

}

impl Server {
    pub fn new(max_connection: usize) -> Server {
        Server { 
            header_size: 16,
            timeout_seconds: 15,
            max_connection: max_connection
        }
    }

    pub fn set_header_size(&mut self, header_size: usize) {
        self.header_size = header_size;
    }

    pub fn set_timeout(&mut self, timeout_seconds: usize) {
        self.timeout_seconds = timeout_seconds;
    }

    pub fn sync_serve<F>(&self, port: u16, handler: F) 
    where 
        F: Fn(&Message) -> Result<Vec<u8>> + Send + Sync + 'static
    {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(self.serve(port, handler));
    }

    pub async fn serve<F>(&self, port: u16, handler: F) 
    where 
        F: Fn(&Message) -> Result<Vec<u8>> + Send + Sync + 'static
    {
        let handler_ref_arc = Arc::new(handler);
        let listener = tokio::net::TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], port)),).await;
        let mut conn_handles: Vec<JoinHandle<()>> = vec![];

        info!("server started on port: {} with max allowed connections: {}!", &port, &self.max_connection);
        loop {
            info!("Currently there are {} active connections", &conn_handles.len());
            match listener.as_ref().unwrap().accept().await {
                Ok((mut stream, addr)) => {
                    Server::manage_connection_handles(&mut conn_handles);

                    let mut stream_wt = tokio_io_timeout::TimeoutReader::new(stream);
                    info!("opening session for client: {}:{}", &addr.ip(), &addr.port());

                    if conn_handles.len() >= self.max_connection {
                        error!("can't accept any more connections, currently {} connection and max is set to {}", &conn_handles.len(), &self.max_connection);
                        stream_wt.get_mut().shutdown().await;
                    } else {
                        let handler_ref = handler_ref_arc.clone();
                        let header_size = self.header_size.clone();
                        let timeout_seconds = self.timeout_seconds.clone();

                        let f = Server::handle_connection(stream_wt, header_size, timeout_seconds, handler_ref);
                        conn_handles.push(tokio::spawn(f));
                    }
                },
                Err(e) => error!("couldn't get client: {e:?}"),
            }
        } 
    }

    fn manage_connection_handles(conn_handles: &mut Vec<JoinHandle<()>>) {
        let futures_idx: Vec<usize> = conn_handles.iter()
        .filter(|f| f.is_finished())
        .enumerate()
        .map(|(i, _)| i)
        .collect();

        for i in futures_idx {
            conn_handles.swap_remove(i);
        }
    }

    async fn handle_connection<F>(mut stream: TimeoutReader<TcpStream>, header_size: usize, timeout: usize, handler: Arc<F>)
    where 
        F: Fn(&Message) -> Result<Vec<u8>> + Send + Sync
    {
        info!("starting session on thread {:?}...", &thread::current().id());

        loop {
            let mut format_buf: Vec<u8> = vec![];

            let response = Server::handle_message(&mut stream, header_size, timeout, handler.clone()).await;
            match response {
                Ok(resp) => {
                    match stream.get_mut().write_all(&resp.to_bytes()[..]).await {
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
    }

    async fn handle_message<F>(stream: &mut TimeoutReader<TcpStream>, header_size: usize, timeout: usize, handler: Arc<F>) -> Result<Response>
    where 
        F: Fn(&Message) -> Result<Vec<u8>> + Send + Sync
    {
        let header_size_less_one_byte = header_size - 1;

        info!("awaiting data...");
        stream.set_timeout(Some(Duration::from_secs(timeout as u64)));

        let mut format_buf: Vec<u8> = vec![];
        let mut peek_buf: Vec<u8> = vec![];
        stream.get_mut().take(1).read_to_end(&mut format_buf).await;
        if format_buf.len() > 0 {
            debug!("detected data");
            stream.set_timeout(Some(Duration::from_secs(5)));
        }
        stream.get_mut().take((Header::size() - 1) as u64).read_to_end(&mut format_buf).await;
        let header = Header::from_bytes(&format_buf)?;
        debug!("completed reading header...");

        let header_string = match str::from_utf8(&format_buf[..]) {
            Ok(ok) => Ok(ok),
            Err(e) => Err(GeneralError::new(e.to_string()))
        };

        if header.message_length == 0 {
            debug!("message size is empty...");
            return Err(GeneralError::new("message size is empty".to_string()));
        } else {
            debug!("message size is: {}", &header.message_length);
        }

        debug!("reading data...");
        let mut data: Vec<u8> = Vec::with_capacity(header.message_length as usize);
        stream.get_mut().read_exact(&mut data[..]).await;
        stream.get_mut().take(header.message_length as u64).read_to_end(&mut data).await;

        debug!("received: {:?}", str::from_utf8(&data));
        let message = Message{header, data};

        match handler(&message) {
            Ok(resp) => Ok(Response::success(resp)),
            Err(e) => Ok(Response::error(format!("{}", e)))
        }
    }    
}