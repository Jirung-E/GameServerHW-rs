use std::net::TcpStream;
use std::io::{Read, Write};
use std::time::SystemTime;


#[derive(serde::Serialize, serde::Deserialize)]
struct Protocol {
    time: u128,
    msg: String,
}


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

                let time = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_millis();
                let msg = Protocol { time, msg };
            
                stream.write(&bincode::serialize(&msg).unwrap()).unwrap();

                let mut data = [0 as u8; 1000]; 
                match stream.read(&mut data) {
                    Ok(_) => {
                        let msg: Protocol = bincode::deserialize(&data).unwrap();
                        let text = msg.msg;
                        let elapsed = SystemTime::now()
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap()
                            .as_millis() - time;
                        println!("[{}ms] Response: {}", elapsed, text);
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
