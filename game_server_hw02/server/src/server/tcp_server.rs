use tokio::{
    net::{TcpListener, TcpStream},
    io::{AsyncReadExt, AsyncWriteExt},
};


pub async fn run_server(ip: &str, port: u16) {
    let addr = format!("{}:{}", ip, port);
    let tcp_listener = TcpListener::bind(addr.clone()).await
        .expect("Failed to bind tcp listener");

    println!("Tcp server - listening on: {}", addr);

    loop {
        let (mut stream, _) = tcp_listener.accept().await
            .expect("Failed to accept connection");

        let mut buf = [0; 1024];
        let n = stream.read(&mut buf).await
            .expect("Failed to read from stream");

        let msg = String::from_utf8_lossy(&buf[..n]);

        handle_message(&mut stream, &msg).await;
    }
}

async fn handle_message(stream: &mut TcpStream, msg: &str) {
    println!("Received message: {}", msg);

    match msg {
        "ping" => {
            let response = "pong";
            stream.write_all(response.as_bytes()).await
                .expect("Failed to write to stream");
        },
        _ => {}
    }
}