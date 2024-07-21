use tokio::net::UdpSocket;

#[tokio::main]
async fn main() {
    let ip = "127.0.0.1";
    let port = 7878;
    let addr = format!("{}:{}", ip, port);
    
    let socket = UdpSocket::bind(addr).await.unwrap();
    
    let mut buf = [0; 1024];
    loop {
        let (len, addr) = socket.recv_from(&mut buf).await.unwrap();
        
        let msg = String::from_utf8_lossy(&buf[..len]);
        println!("Received: {}", msg);

        let response = handle_message(&msg);

        socket.send_to(response.as_bytes(), &addr).await.unwrap();
    }
}


fn handle_message(msg: &str) -> &str {
    match msg {
        "Hello" => "Hi",
        _ => "Unknown",
    }
}
