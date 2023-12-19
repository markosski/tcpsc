use std::io::prelude::*;
use std::net::TcpStream;
use std::time::Duration;
use std::io::BufReader;
use std::io::{stdin,stdout,Write};

fn main() {
    if let Ok(mut stream) = TcpStream::connect("127.0.0.1:7878") {
        println!("Connected to the server!");

        loop {
            print!("Enter text to send: ");
            
            let mut data = String::new();
            let _ = stdout().flush();
            stdin().read_line(&mut data).expect("Did not enter a correct string");
            data += "\n"; // new line to indicate EOF

            match stream.write_all(&data.as_bytes()) {
                Ok(_) => println!("data sent: {:?}", &data.as_bytes()),
                Err(err) => println!("error responding to server; {}", err.to_string())
            };

            let _ = stream.set_read_timeout(Some(Duration::new(3, 0)));

            let buf_reader = BufReader::new(&mut stream);
            let response: Vec<_> = buf_reader
                .lines()
                .map(|result| result.unwrap())
                .take_while(|line| !line.is_empty())
                .collect();

            println!("received: {:?}", &response);
        }
    } else {
        println!("Couldn't connect to server...");
    }
}
