use std::collections::HashMap;
use tokio::sync::mpsc;


struct Player {
    x: i32,
    y: i32,
}


pub type WorldPointer = usize;


pub struct World {
    players: HashMap<u32, Player>,
    sender: mpsc::Sender<String>, 
    receiver: mpsc::Receiver<String>,
}

impl World {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel(128);
        Self {
            players: HashMap::new(),
            sender,
            receiver,
        }
    }

    pub async fn run_message_loop(&mut self) {
        loop {
            match self.receiver.recv().await {
                Some(msg) => {
                    println!("channel received: {}", msg);

                    let msg = msg.split_whitespace()
                        .collect::<Vec<&str>>();
                
                    match msg[0] {
                        "add" => {
                            let id = msg[1].parse::<u32>().unwrap();
                            self.add_player(id);
                        },
                        
                        "move" => {
                            let id = msg[1].parse::<u32>().unwrap();
                            let x = msg[2].parse::<i32>().unwrap();
                            let y = msg[3].parse::<i32>().unwrap();
                            self.move_player(id, x, y);
                        },
                
                        "remove" => {
                            let id = msg[1].parse::<u32>().unwrap();
                            self.remove_player(id);
                        },
                
                        _ => {}
                    }
                },
                None => {
                    println!("channel closed");
                    break;
                }
            }
        }
    }


    pub fn add_player(&mut self, id: u32) {
        self.players.insert(id, Player { x: 3, y: 3 });
    }

    pub fn move_player(&mut self, id: u32, x: i32, y: i32) {
        println!("Move {}: ({}, {})", id, x, y);

        if let Some(player) = self.players.get_mut(&id) {
            player.x += x;
            player.y += y;

            if player.x < 0 {
                player.x = 0;
            }
            if player.y < 0 {
                player.y = 0;
            }
            if player.x > 7 {
                player.x = 7;
            }
            if player.y > 7 {
                player.y = 7;
            }
        }
    }

    pub fn remove_player(&mut self, id: u32) {
        self.players.remove(&id);
    }

    pub fn update_message(&self) -> String {
        let objects = self.players.iter()
            .map(|(id, player)| format!("{} {} {}", id, player.x, player.y))
            .collect::<Vec<String>>();
        
        format!("update {} {}", objects.len(), objects.join(" "))
    }
}

impl Into<WorldPointer> for &World {
    fn into(self) -> WorldPointer {
        self as *const World as WorldPointer
    }
}



/// Mutex를 적용하면 read할때도 lock을 걸어야 하기 때문에 사용하지 않음.
/// `Arc`를 사용해서 여러 스레드에서 **read**가능하도록 하고,
/// **write**이 필요한 경우는 `WorldInterface`에서 `mpsc`를 통해 `World`로 메세지를 보내서 처리.
pub struct WorldInterface {
    world: WorldPointer,
    sender: mpsc::Sender<String>,
}

impl WorldInterface {
    pub fn new(world: WorldPointer) -> Self {
        let ptr = world as *const World ;

        Self { 
            world: world as WorldPointer, 
            sender: unsafe { &*ptr }.sender.clone(),
        }
    }

    pub async fn add_player(&self, id: u32) {
        self.sender.send(format!("add {}", id)).await.unwrap();
    }

    pub async fn move_player(&self, id: u32, x: i32, y: i32) {
        self.sender.send(format!("move {} {} {}", id, x, y)).await.unwrap();
    }

    pub async fn remove_player(&self, id: u32) {
        self.sender.send(format!("remove {}", id)).await.unwrap();
    }

    pub fn update_message(&self) -> String {
        unsafe { &*(self.world as *const World) }.update_message()
    }
}
