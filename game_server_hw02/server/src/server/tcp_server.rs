use tokio::{
    net::{TcpListener, TcpStream},
    io::{AsyncReadExt, AsyncWriteExt},
};


struct Player {
    id: u32,
    x: i32,
    y: i32,
}

static mut PLAYER: Player = Player { id: 0, x: 3, y: 3 };


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
    let mut buf = [0; 1024];

    send_message(&mut stream, &update_message()).await;
    
    loop {
        let response = match stream.read(&mut buf).await {
            Ok(0) => {
                println!("Connection closed");
                return;
            },
            Ok(n) => {
                let msg = String::from_utf8_lossy(&buf[..n]);
                process_message(&msg).await
            },
            Err(e) => {
                eprintln!("Failed to read from socket; err = {:?}", e);
                return;
            }
        };

        if let Some(response) = response {
            send_message(&mut stream, &response).await;
        }
    }
}

async fn process_message(msg: &str) -> Option<String> {
    println!("Received message: {}", msg);

    let msg = msg.trim().split_whitespace().collect::<Vec<&str>>();

    if msg.len() == 0 {
        return None;
    }

    match msg[0] {
        "ping" => {
            Some("pong".to_string())
        },

        "move" => {
            if msg.len() != 4 {
                return None;
            }

            let id = msg[1].parse::<u32>().unwrap();
            let x = msg[2].parse::<i32>().unwrap();
            let y = msg[3].parse::<i32>().unwrap();

            println!("Move {}: ({}, {})", id, x, y);

            unsafe {
                PLAYER.x += x;
                PLAYER.y += y;

                if PLAYER.x < 0 {
                    PLAYER.x = 0;
                }
                if PLAYER.y < 0 {
                    PLAYER.y = 0;
                }
                if PLAYER.x > 7 {
                    PLAYER.x = 7;
                }
                if PLAYER.y > 7 {
                    PLAYER.y = 7;
                }
            }

            None
        },

        "update" => {
            Some(update_message())
        }

        _ => None
    }
}

fn update_message() -> String {
    let num_objects = 1usize;
    let objects = vec![format!("{} {} {}", unsafe { PLAYER.id }, unsafe { PLAYER.x }, unsafe { PLAYER.y })]
        .join(" ");

    format!("update {} {}", num_objects, objects)
}

async fn send_message(stream: &mut TcpStream, msg: &str) {
    let msg = format!("GAMESERVER {}\n", msg);
    stream.write_all(msg.as_bytes()).await
        .expect("Failed to write to socket");
}