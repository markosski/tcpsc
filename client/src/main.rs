use std::io::prelude::*;
use std::net::TcpStream;
use std::time::Duration;
use std::io::BufReader;
use std::io::{stdin,stdout,Write};

fn main() -> std::io::Result<()> {
    if let Ok(mut stream) = TcpStream::connect("127.0.0.1:7878") {
        println!("Connected to the server!");

        loop {
            print!("Please enter text to send: ");
            
            let mut data = String::new();
            let _ = stdout().flush();
            stdin().read_line(&mut data).expect("Did not enter a correct string");
            data += "\r\n";

            stream.write_all(&data.as_bytes())?;
            println!("data sent: {}", &data);

            // let _ = stream.set_read_timeout(Some(Duration::new(2, 0)));
            let buf_reader = BufReader::new(&mut stream);
            let response: Vec<_> = buf_reader
                .lines()
                .map(|result| result.unwrap())
                .take_while(|line| !line.is_empty())
                .collect();

            stream.flush()?;
            println!("received: {}", response.join("\n"));
        }
    } else {
        println!("Couldn't connect to server...");
    }

    Ok(())
}
