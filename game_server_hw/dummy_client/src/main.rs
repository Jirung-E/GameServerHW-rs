#![allow(dead_code)]

use std::{
    rc::Rc, 
    cell::RefCell, 
    collections::HashMap, 
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::{
    net::TcpStream,
    io::{AsyncReadExt, AsyncWriteExt},
};
use rand::Rng;
use get_addr::get_addr;
use network::*;


struct Player {
    x: i32,
    z: i32,
}

struct Server {
    players: HashMap<u32, Rc<RefCell<Player>>>,

    player_id: u32,

    addr: String,
    stream: TcpStream,
    packet_parser: PacketParser,

    timer: SystemTime,
}

impl Server {
    pub async fn new() -> Self {
        let (ip, port) = match get_addr() {
            Ok((ip, port)) => (ip, port),
            Err(e) => panic!("{}", e),
        };
        
        let addr = format!("{}:{}", ip, port);
        let stream = TcpStream::connect(addr.clone()).await.unwrap();

        Self {
            players: HashMap::new(),

            player_id: 0,

            addr,
            stream,
            packet_parser: PacketParser::new(),

            timer: SystemTime::now(),
        }
    }

    fn player(&self) -> Option<Rc<RefCell<Player>>> {
        self.players.get(&self.player_id).cloned()
    }

    async fn pull_messages(&mut self) {
        let mut buf = [0; 1024];

        match self.stream.read(&mut buf).await {
            Ok(0) => println!("Connection closed"),
            
            Ok(n) => self.packet_parser.push(&buf[..n]),

            Err(e) => eprintln!("Failed to read from socket; err = {:?}", e),
        }
    }
    
    fn process_message(&mut self, msg: &str) {
        let msg = msg.trim().split_whitespace()
            .map(|s| s.trim())
            .collect::<Vec<&str>>();

        // println!("Received: {:?}", msg);

        if msg.len() == 0 {
            return;
        }

        match msg[0] {
            "init" => {
                if msg.len() < 2 {
                    return;
                }

                self.player_id = msg[1].parse::<u32>().unwrap();
            }

            "update" => {
                if msg.len() < 2 {
                    return;
                }

                let num_objects = msg[1].parse::<usize>().unwrap();
                let mut valid_ids: Vec<u32> = Vec::new();

                for i in 0..num_objects {
                    let idx = 2 + i * 3;
                    let id = msg[idx].parse::<u32>().unwrap();
                    let x = msg[idx+1].parse::<i32>().unwrap();
                    let z = msg[idx+2].parse::<i32>().unwrap();

                    self.players.entry(id)
                        .or_insert_with(|| {
                            Rc::new(RefCell::new(Player { x, z }))
                        });
                    
                    valid_ids.push(id);
                }

                // 기존에 있던 id가 안보이면 삭제
                self.players.retain(|k, _| valid_ids.contains(k));
            }
            _ => {}
        }
    }

    async fn update(&mut self) {
        let now = std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH).unwrap()
            .as_millis();
        let packet = MessagePacket::new(now, "update");

        self.stream.write_all(&packet.as_bytes()).await
            .expect("Failed to write to stream");

        self.pull_messages().await;

        while let Some(packet) = self.packet_parser.pop() {
            let packet = match MessagePacket::from_raw(packet) {
                Ok(packet) => packet,
                Err(_) => continue,
            };
            
            let msg = packet.msg();
            self.process_message(&msg);

            if self.timer.elapsed().unwrap().as_secs() >= 1 {
                self.timer = SystemTime::now();
                let latency = self.timer
                    .duration_since(UNIX_EPOCH).unwrap()
                    .as_millis() - packet.time();
                println!("server {} latency: {}ms", self.player_id, latency);
            }
        }

        if self.timer.elapsed().unwrap().as_millis() >= 1000 {
            let mut rng = rand::thread_rng();
            let (x, z) = match rng.gen_range(0..4) {
                0 => (1, 0),
                1 => (0, 1),
                2 => (-1, 0),
                3 => (0, -1),
                _ => (0, 0),
            };

            let move_msg = format!("move {} {x} {z}\n", self.player_id);
            let packet = MessagePacket::new(now, &move_msg);
            self.stream.write_all(&packet.as_bytes()).await
                .expect("Failed to write to stream");
        }
    }
}



use futures::future::join_all;

#[tokio::main]
async fn main() {
    let servers = (0..1000).map(|_| new_server());
    join_all(servers).await;

    println!("done");
}

async fn new_server() {
    let mut server = Server::new().await;

    loop {
        server.update().await;
    }
}