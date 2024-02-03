use std::io::{stdin,stdout,Write};
use std::env;
use core::models::{Header, ResponseType};
use core::utils;
use core::client::Client;
extern crate env_logger;

fn main() {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Please provide port number");
    }
    let port: u16 = args[1].parse().expect("valid port number");

    if let Ok(mut client) = Client::new([127,0,0,1], port, 3) {
        loop {
            print!("Enter text to send: ");
            let mut data = String::new();
            let _ = stdout().flush();
            stdin().read_line(&mut data).expect("Did not enter a correct string");
            let data_bytes = data.into_bytes();

            let msg_type: [u8; 4] = utils::to_array::<4>("REQ ");
            let msg_client_id: [u8; 8] = utils::to_array::<8>("abc12345");
            let header = Header::new(msg_type, msg_client_id, data_bytes.len() as u32);

            match client.send(header, data_bytes) {
                Ok(response) => {
                    match response.response_type {
                        ResponseType::SUCC => println!("{:?}", &response.success_as_string()),
                        ResponseType::ERR => println!("{:?}", &response.error_as_string())
                    }
                },
                Err(e) => println!("{}", e.to_string())
            }
        }
    }
}