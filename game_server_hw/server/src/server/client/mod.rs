use tokio::{
    net::TcpStream,
    io::{AsyncReadExt, AsyncWriteExt},
};
use super::world::WorldInterface;
use network::*;


pub struct Client {
    id: u32,
    
    stream: TcpStream,
    packet_parser: PacketParser,

    world: WorldInterface,

    running: bool,
}

impl Client {
    pub fn new(id: u32, stream: TcpStream, world: WorldInterface) -> Self {
        Self {
            id,
            stream,
            packet_parser: PacketParser::new(),
            world,
            running: true,
        }
    }

    pub async fn handle_connection(&mut self) {
        self.world.add_player(self.id).await;
        self.stream_write(&format!("init {}", self.id)).await;

        let mut buf = [0; 1024];
    
        while self.running {
            let read = self.stream.read(&mut buf).await;
    
            match read {
                Ok(0) => {
                    println!("Connection closed");
                    break;
                },
    
                Ok(n) => {
                    self.process_packets(&buf[..n]).await;
                },
    
                Err(e) => {
                    eprintln!("Failed to read from socket; err = {:?}", e);
                    break;
                },
            };
        }

        self.world.remove_player(self.id).await;
    }

    
    async fn process_packets(&mut self, data: &[u8]) {
        self.packet_parser.push(data);

        while let Some(packet) = self.packet_parser.pop() {
            let msg = String::from_utf8_lossy(&packet);

            if let Some(response) = self.process_message(&msg).await {
                self.stream_write(&response).await;
            }
        }
    }

    async fn process_message(&mut self, msg: &str) -> Option<String> {
        let msg = msg.trim().split_whitespace()
            .collect::<Vec<&str>>();
    
        if msg.len() == 0 {
            return None;
        }
    
        match msg[0] {
            "ping" => Some("pong".to_string()),
    
            "move" if msg.len() == 4 => {
                let id = msg[1].parse::<u32>().unwrap();
                let x = msg[2].parse::<i32>().unwrap();
                let y = msg[3].parse::<i32>().unwrap();
    
                self.world.move_player(id, x, y).await;
    
                None
            },
    
            "update" => Some(self.world.update_message()),
    
            _ => None
        }
    }

    async fn stream_write(&mut self, msg: &str) {
        let msg = format!("GAMESERVER {}\n", msg);
        
        match self.stream.write_all(msg.as_bytes()).await {
            Ok(_) => {},
            Err(e) => {
                eprintln!("Failed to write to socket; err = {:?}", e);
                self.running = false;
            }
        }
    }
}