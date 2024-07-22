use std::thread;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{Read, Write};
use std::str::from_utf8;

fn handle_client(mut stream: TcpStream) {
    let mut data = [0 as u8; 1000]; 
    while match stream.read(&mut data) {
        Ok(_size) => {
            let text = from_utf8(&data).unwrap();
            print!("[{}] : {}", stream.peer_addr().unwrap(), text);
            stream.write(text.as_bytes()).unwrap();
            true
        },
        Err(_) => {
            println!("Disconnect [{}]", stream.peer_addr().unwrap());
            stream.shutdown(Shutdown::Both).unwrap();
            false
        }
    } {}
}

fn main() {
    println!("[Server]");

    println!("Enter server ip");
    let mut ip = String::from("");
    std::io::stdin().read_line(&mut ip).unwrap();
    let server_ip = &ip[0..ip.chars().count()-2];
    
    println!("Enter server port");
    let mut port = String::from("");
    std::io::stdin().read_line(&mut port).unwrap();
    let server_port = &port[0..port.chars().count()-2];

    let mut server_address = String::from("");
    server_address.push_str(server_ip);
    server_address.push_str(":");
    server_address.push_str(server_port);

    let listener = TcpListener::bind(server_address).unwrap();
    println!("Server listening on port {server_port}");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New Connect [{}]", stream.peer_addr().unwrap());
                thread::spawn(move|| {
                    handle_client(stream)
                });
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
    drop(listener);
}
