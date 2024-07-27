use tokio::{
    net::TcpStream,
    io::{AsyncReadExt, AsyncWriteExt},
};
use super::world::WorldInterface;


pub struct Client {
    id: u32,
    stream: TcpStream,
    world: WorldInterface,
}

impl Client {
    pub fn new(id: u32, stream: TcpStream, world: WorldInterface) -> Self {
        Self {
            id,
            stream,
            world,
        }
    }

    pub async fn handle_connection(&mut self) {
        self.world.add_player(self.id).await;
        self.stream_write(&format!("init {}", self.id)).await;

        let mut buf = [0; 1024];
    
        loop {
            let read = self.stream.read(&mut buf).await;
    
            match read {
                Ok(0) => {
                    println!("Connection closed");
                    break;
                },
    
                Ok(n) => {
                    let msg = String::from_utf8_lossy(&buf[..n]);
                    self.process_messages(&msg).await;
                },
    
                Err(e) => {
                    eprintln!("Failed to read from socket; err = {:?}", e);
                    break;
                },
            };
        }

        self.world.remove_player(self.id).await;
    }

    pub async fn process_messages(&mut self, msg: &str) {
        let messages = msg.trim().split("\n")
            .collect::<Vec<&str>>();

        for msg in messages {
            if let Some(response) = self.process_message(msg).await {
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

    pub async fn stream_write(&mut self, msg: &str) {
        let msg = format!("GAMESERVER {}\n", msg);
        self.stream.write_all(msg.as_bytes()).await
            .expect("Failed to write to socket");
    }
}