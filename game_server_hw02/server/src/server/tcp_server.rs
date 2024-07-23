use tokio::{
    net::{TcpListener, TcpStream},
    io::{AsyncReadExt, AsyncWriteExt},
};


struct Player {
    id: u32,
    x: i32,
    y: i32,
}

const ARRAY_REPEAT_VALUE: Option<Player> = None;
static mut PLAYERS: [Option<Player>; 10] = [ARRAY_REPEAT_VALUE; 10];


pub async fn run_server(ip: &str, port: u16) {
    let addr = format!("{}:{}", ip, port);
    let tcp_listener = TcpListener::bind(addr.clone()).await
        .expect("Failed to bind tcp listener");

    println!("Tcp server - listening on: {}", addr);

    loop {
        println!("Waiting for connection...");
        match tcp_listener.accept().await {
            Ok((stream, addr)) => {
                println!("Accepted connection from: {}", addr);
                tokio::spawn(handle_connection(stream));
            },
            Err(e) => {
                eprintln!("Failed to accept connection; err = {:?}", e);
            }
        }
    }
}

/// 초기화 메시지 송신
/// 
/// loop {
///     메시지 수신
///     if 메시지 키워드 일치 {
///         메시지 처리
///         응답 메시지 송신
///     }
/// }
async fn handle_connection(mut stream: TcpStream) {
    let id = match add_player() {
        // join
        Ok(id) => {
            println!("Player added");
            id
        }

        // kick
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };

    send_message(&mut stream, &format!("init {}", id)).await;
    send_message(&mut stream, &update_message()).await;
    
    let mut buf = [0; 1024];
    
    loop {
        let read = stream.read(&mut buf).await;
        
        // print!("{} - ", stream.peer_addr().unwrap());
        // use std::io::Write;
        // std::io::stdout().flush().unwrap();

        match read {
            Ok(0) => {
                println!("Connection closed");
                unsafe {
                    PLAYERS[id as usize] = None;
                }
                return;
            },

            Ok(n) => {
                let msg = String::from_utf8_lossy(&buf[..n]);
                // println!("Received: {}", msg);
                process_messages(&mut stream, &msg).await;
            },

            Err(e) => {
                eprintln!("Failed to read from socket; err = {:?}", e);
                unsafe {
                    PLAYERS[id as usize] = None;
                }
                return;
            },
        };
    }
}

async fn process_messages(stream: &mut TcpStream, msg: &str) {
    let messages = msg.trim().split("\n").collect::<Vec<&str>>();

    for msg in messages {
        if let Some(response) = process_message(msg) {
            send_message(stream, &response).await;
        }
    }
}

fn process_message(msg: &str) -> Option<String> {
    let msg = msg.trim().split_whitespace().collect::<Vec<&str>>();

    if msg.len() == 0 {
        return None;
    }

    match msg[0] {
        "ping" => Some("pong".to_string()),

        "move" if msg.len() == 4 => {
            let id = msg[1].parse::<u32>().unwrap();
            let x = msg[2].parse::<i32>().unwrap();
            let y = msg[3].parse::<i32>().unwrap();

            move_player(id, x, y);

            None
        },

        "update" => Some(update_message()),

        _ => None
    }
}

fn add_player() -> Result<u32, String> {
    unsafe {
        for i in 0..PLAYERS.len() {
            if PLAYERS[i].is_none() {
                PLAYERS[i] = Some(Player {
                    id: i as u32,
                    x: 3,
                    y: 3,
                });
                return Ok(i as u32);
            }
        }
    }

    Err("Failed to add player".to_string())
}

fn move_player(id: u32, x: i32, y: i32) {
    println!("Move {}: ({}, {})", id, x, y);

    unsafe {
        if let Some(player) = PLAYERS[id as usize].as_mut() {
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
}

fn update_message() -> String {
    let objects = unsafe {
        PLAYERS.iter()
            .filter_map(|player| player.as_ref())
            .map(|player| format!("{} {} {}", player.id, player.x, player.y))
            .collect::<Vec<String>>()
    };
    
    format!("update {} {}", objects.len(), objects.join(" "))
}

async fn send_message(stream: &mut TcpStream, msg: &str) {
    let msg = format!("GAMESERVER {}\n", msg);
    stream.write_all(msg.as_bytes()).await
        .expect("Failed to write to socket");
}