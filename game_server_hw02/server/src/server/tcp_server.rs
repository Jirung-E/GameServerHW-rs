use tokio::{
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

async fn handle_connection(mut stream: TcpStream) {
    {
        let init_msg = format!("set {} {}\n", unsafe { PLAYER.x }, unsafe { PLAYER.y });
        stream.write_all(init_msg.as_bytes()).await
            .expect("Failed to write to socket");
    }

    let mut buf = [0; 1024];
    
    loop {
        match stream.read(&mut buf).await {
            Ok(0) => {
                println!("Connection closed");
                return;
            },
            Ok(n) => {
                let msg = String::from_utf8_lossy(&buf[..n]);
                if let Some(response) = handle_message(&msg).await {
                    stream.write_all(response.as_bytes()).await
                        .expect("Failed to write to socket");
                }
            },
            Err(e) => {
                eprintln!("Failed to read from socket; err = {:?}", e);
            }
        }
    }
}

async fn handle_message(msg: &str) -> Option<String> {
    println!("Received message: {}", msg);

    let msg = msg.trim().split_whitespace().collect::<Vec<&str>>();

    if msg.len() == 0 {
        return None;
    }

    match msg[0] {
        "ping" => {
            Some("pong\n".to_string())
        },

        "move" => {
            if msg.len() != 3 {
                return None;
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

            Some(format!("set {} {}\n", unsafe { PLAYER.x }, unsafe { PLAYER.y }))
        },

        _ => None
    }
}