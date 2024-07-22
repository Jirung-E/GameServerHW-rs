use futures::future;
use tokio::{
    select,
    net::{TcpListener, TcpStream},
    io::{AsyncReadExt, AsyncWriteExt},
};


struct Player {
    x: i32,
    y: i32,
}

static mut PLAYER: Player = Player { x: 3, y: 3 };


pub async fn run_server(ip: &str, port: u16) {
    let addr = format!("{}:{}", ip, port);
    let tcp_listener = TcpListener::bind(addr.clone()).await
        .expect("Failed to bind tcp listener");

    println!("Tcp server - listening on: {}", addr);

    // 게임 플레이 중에는 참가/퇴장의 빈도가 높지 않을것이므로 LinkedList 대신 Vec 사용
    let mut streams: Vec<TcpStream> = Vec::new();

    loop {
        if streams.len() == 0 {
            println!("Waiting for connection...");
            if let Ok((mut stream, addr)) = tcp_listener.accept().await {
                println!("Accepted connection from: {}", addr);
                let response = format!("set {} {}\n", unsafe { PLAYER.x }, unsafe { PLAYER.y });
                stream.write_all(response.as_bytes()).await
                    .expect("Failed to write to stream");
                streams.push(stream);
            }
        }
        else {
            select! {
                accept_result = tcp_listener.accept() => {
                    if let Ok((mut stream, addr)) = accept_result {
                        println!("Accepted connection from: {}", addr);
                        let response = format!("set {} {}\n", unsafe { PLAYER.x }, unsafe { PLAYER.y });
                        stream.write_all(response.as_bytes()).await
                            .expect("Failed to write to stream");
                        streams.push(stream);
                    }
                }
                _ = future::ready(()) => {}
            }
        }
        
        let mut buf = [0; 1024];

        let mut invalid_indices: Vec<usize> = Vec::new();
        
        for (i, stream) in streams.iter_mut().enumerate() {
            let read_result = stream.read(&mut buf).await;
            match read_result {
                Ok(0) => {
                    println!("Connection closed");
                    invalid_indices.push(i);
                },
                Ok(n) => {
                    let msg = String::from_utf8_lossy(&buf[..n]);
                    handle_message(stream, &msg).await;
                },
                Err(e) => {
                    eprintln!("Failed to read from socket; err = {:?}", e);
                    invalid_indices.push(i);
                }
            }
        }

        for i in invalid_indices.iter().rev() {
            streams.swap_remove(*i);
        }
    }
}

async fn handle_message(stream: &mut TcpStream, msg: &str) {
    println!("Received message: {}", msg);

    let msg = msg.trim().split_whitespace().collect::<Vec<&str>>();

    if msg.len() == 0 {
        return;
    }

    match msg[0] {
        "ping" => {
            let response = "pong\n";
            stream.write_all(response.as_bytes()).await
                .expect("Failed to write to stream");
        },

        "move" => {
            if msg.len() != 3 {
                return;
            }

            let x = msg[1].parse::<i32>().unwrap_or(0);
            let y = msg[2].parse::<i32>().unwrap_or(0);

            println!("Move: ({}, {})", x, y);

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

            let response = format!("set {} {}\n", unsafe { PLAYER.x }, unsafe { PLAYER.y });
            stream.write_all(response.as_bytes()).await
                .expect("Failed to write to stream");
        },

        _ => {}
    }
}