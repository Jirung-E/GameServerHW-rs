use std::net::TcpStream;
use std::io::{Read, Write};
use std::str::from_utf8;

fn main() {
    println!("[Client]");
    println!("Enter server address (ip:port)");
    let mut address = String::from("");
    let _temp = std::io::stdin().read_line(&mut address).unwrap();
    let server_address = &address[0..address.chars().count()-2];

    match TcpStream::connect(server_address) {
        Ok(mut stream) => {
            println!("Successfully connected to server in address {}\n", server_address);
            loop {
                let mut msg = String::from("");
                std::io::stdin().read_line(&mut msg).unwrap();
            
                stream.write(msg.as_bytes()).unwrap();

                let mut data = [0 as u8; 1000]; 
                match stream.read(&mut data) {
                    Ok(_) => {
                        if &data == msg.as_bytes() {
                            let text2 = from_utf8(&data).unwrap();
                            print!("recv : {text2}end");
                        } else {
                            let text = from_utf8(&data).unwrap();
                            println!("[echo server] : {}", text);
                        }
                    },
                    Err(e) => {
                        println!("Failed to receive data: {}", e);
                    }
                }
            }
        },
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }
    println!("Terminated.");
}
