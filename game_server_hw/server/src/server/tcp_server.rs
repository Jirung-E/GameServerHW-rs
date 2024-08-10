use tokio::net::{TcpListener, TcpStream};
use std::sync::Mutex;

use super::{
    world::*,
    client::Client,
};



pub async fn run_server(ip: &str, port: u16) {
    let addr = format!("{}:{}", ip, port);
    let tcp_listener = TcpListener::bind(addr.clone()).await
        .expect("Failed to bind tcp listener");

    println!("Tcp server - listening on: {}", tcp_listener.local_addr().unwrap());

    let mut world = World::new();

    {
        tokio::spawn(wait_for_players(tcp_listener, (&world).into()));
    }

    world.run_message_loop().await; 
}


const MAX_CLIENTS: usize =  10000;
/// World를 직접 읽으면 최신 데이터가 아닐 가능성이 있다.  
/// World에 Mutex, RwLock등을 걸면 클라이언트가 읽는데 병목이 생길 수 있다.  
/// 따라서 클라이언트 개수만 세기 위해 따로 분리.  
static CLIENT_SLOTS: Mutex<[Option<()>; MAX_CLIENTS]> = Mutex::new([None; MAX_CLIENTS]);



/// Listens for incoming connections
async fn wait_for_players(listener: TcpListener, world: WorldPointer) {
    loop {
        // println!("Waiting for connection...");
        match listener.accept().await {
            Ok((stream, _addr)) => {
                let mut slots = CLIENT_SLOTS.lock().unwrap();
                let mut accepted = false;

                for id in 0..MAX_CLIENTS {
                    if slots[id].is_none() {
                        slots[id] = Some(());
                        // println!("Accepted connection from: {}", addr);
                        accepted = true;
                        tokio::spawn(handle_connection(id as u32, stream, world));
                        break;
                    }
                }
                if !accepted {
                    // println!("Connection from {} refused; server full", addr);
                }
            },
            Err(_) => {
                // eprintln!("Failed to accept connection; err = {:?}", e);
            }
        }
    }
}


async fn handle_connection(id: u32, stream: TcpStream, world: WorldPointer) {
    let mut client = Client::new(id, stream, WorldInterface::new(world));

    {
        let slots = CLIENT_SLOTS.lock().unwrap();
        println!("num clients: {}", slots.iter().filter(|x| x.is_some()).count());
    }
    
    client.handle_connection().await;

    {
        let mut slots = CLIENT_SLOTS.lock().unwrap();
        slots[id as usize] = None;
        println!("Connection {} closed", id);

        println!("num clients: {}", slots.iter().filter(|x| x.is_some()).count());
    }
}
