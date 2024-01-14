use std::io::{stdin,stdout,Write};
use std::env;
use core::models::Header;
use core::utils;
use core::client::Client;
use std::str;

fn main() {
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
            let msg_client_id: [u8; 8] = utils::to_array::<8>("abc12300");
            let header = Header::new(msg_type, msg_client_id, data_bytes.len() as u32);

            match client.send(header, data_bytes) {
                Ok(resp) => println!("{:?}", str::from_utf8(&resp)),
                Err(e) => println!("{}", e.to_string())
            }
        }
    }
}