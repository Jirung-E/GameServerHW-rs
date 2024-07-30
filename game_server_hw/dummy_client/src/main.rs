#![allow(dead_code)]

use std::{
    rc::Rc, 
    cell::RefCell, 
    io::{Read, Write}, 
    net::TcpStream,
    collections::HashMap,
};
use get_addr::get_addr;


struct Player {
    x: i32,
    z: i32,
}

struct Server {
    players: HashMap<u32, Rc<RefCell<Player>>>,

    player_id: u32,

    addr: String,
    stream: TcpStream,

    prev_update: std::time::Instant,
}

impl Server {
    pub fn new() -> Self {
        let (ip, port) = match get_addr() {
            Ok((ip, port)) => (ip, port),
            Err(e) => {
                panic!("{}", e);
            }
        };
        // let ip = "127.0.0.1".to_string();
        // let port = 8080;
        let addr = format!("{}:{}", ip, port);
        let stream = TcpStream::connect(addr.clone()).unwrap();
        stream.set_nonblocking(true).unwrap();

        Self {
            players: HashMap::new(),

            player_id: 0,

            addr,
            stream,

            prev_update: std::time::Instant::now(),
        }
    }

    fn player(&self) -> Option<Rc<RefCell<Player>>> {
        self.players.get(&self.player_id).cloned()
    }

    fn pull_messages(&mut self) -> Option<String> {
        let mut buf = [0; 1024];

        match self.stream.read(&mut buf) {
            Ok(0) => {
                println!("Connection closed");
                None
            },
            Ok(n) => {
                let msg = String::from_utf8_lossy(&buf[..n]);
                // println!("Received: {}", msg);
                Some(msg.to_string())
            },
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                None
            },
            Err(e) => {
                eprintln!("Failed to read from socket; err = {:?}", e);
                match TcpStream::connect(self.addr.clone()) {
                    Ok(stream) => {
                        self.stream = stream;
                        self.stream.set_nonblocking(true).unwrap();
                        self.pull_messages()
                    },
                    Err(e) => {
                        eprintln!("Failed to reconnect; err = {:?}", e);
                        None
                    }
                }
            }
        }
    }

    fn process_messages(&mut self, msg: &str) {
        let messages = msg.trim().split("\n").collect::<Vec<&str>>();

        // println!("messages: {:?}", messages);

        for msg in messages {
            self.process_message(msg);
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

        if msg[0] != "GAMESERVER" {
            return;
        }

        let msg = &msg[1..];

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

                let time = std::time::Instant::now();
                let elapsed = time.duration_since(self.prev_update);
                println!("Elapsed(id: {}): {:?}", self.player_id, elapsed);
                self.prev_update = time;

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

    fn update(&mut self) {
        self.stream.write_all(b"update\n")
            .expect("Failed to write to stream");

        while let Some(msg) = self.pull_messages() {
            self.process_messages(&msg);
        }
    }
}



#[tokio::main]
async fn main() {
    for _ in 0..10 {
        tokio::spawn(async {
            new_server();
        });
    }

    loop {
        
    }
}

fn new_server() {
    let mut server = Server::new();

    loop {
        server.update();
    }
}