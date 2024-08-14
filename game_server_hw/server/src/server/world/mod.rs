use std::collections::HashMap;


struct Player {
    x: i32,
    y: i32,
}


pub type WorldPointer = usize;


pub struct World {
    players: HashMap<u32, Player>,
}

impl World {
    pub fn new() -> Self {
        Self {
            players: HashMap::new(),
        }
    }


    pub fn add_player(&mut self, id: u32) {
        self.players.insert(id, Player { x: 3, y: 3 });
    }

    pub fn move_player(&mut self, id: u32, x: i32, y: i32) {
        // println!("Move {}: ({}, {})", id, x, y);

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
pub struct WorldInterface {
    /// raw pointer가 future간 이동이 안돼서, usize타입으로 변환하여 사용
    world: WorldPointer,
}

impl WorldInterface {
    pub fn new(world: WorldPointer) -> Self {
        Self { 
            world: world as WorldPointer, 
        }
    }

    /// id가 겹치지 않음을 사용하는쪽에서 보장해야 함.
    pub async fn add_player(&self, id: u32) {
        self.as_mut().add_player(id);
    }

    pub async fn move_player(&self, id: u32, x: i32, y: i32) {
        self.as_mut().move_player(id, x, y);
    }

    /// #### update_message와 동시 호출테스트
    /// dummy_client가 터진다?
    pub async fn remove_player(&self, id: u32) {
        self.as_mut().remove_player(id);
    }

    pub fn update_message(&self) -> String {
        self.as_mut().update_message()
    }

    fn as_mut(&self) -> &mut World {
        unsafe { 
            &mut *(self.world as *mut World)
        }
    }
}
