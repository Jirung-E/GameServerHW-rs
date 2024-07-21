use tokio::net::UdpSocket;


pub async fn run_server(ip: &str, port: u16) {
    let addr = format!("{}:{}", ip, port);
    let udp_socket = UdpSocket::bind(addr.clone()).await
        .expect("Failed to bind udp socket");

    println!("Udp server - listening on: {}", addr);

    loop {
        let mut buf = [0; 1024];
        let (amt, src) = udp_socket.recv_from(&mut buf).await
            .expect("Failed to receive from socket");

        let msg = String::from_utf8_lossy(&buf[..amt]);
        println!("Received: {}", msg);

        handle_message(&udp_socket, &src, &msg).await;
    }
}

async fn handle_message(socket: &UdpSocket, src: &std::net::SocketAddr, msg: &str) {
    println!("Received message from: {}", src);

    match msg {
        "ping" => {
            let response = "pong";
            socket.send_to(response.as_bytes(), src).await
                .expect("Failed to send response");
        },
        _ => {}
    }
}