use tokio::net::UdpSocket;
use lockfree::queue::Queue;
use std::sync::Arc;


/// 중앙 서버와 통신하는 구조체
pub struct Server {
    ip: String,     // 중앙 서버 IP
    port: u16,      // 중앙 서버 포트
    incoming: Arc<Queue<String>>,
    outgoing: Arc<Queue<String>>,
}

impl Server {
    pub async fn new(ip: &str, port: u16) -> Option<Self> {
        Some(Self { 
            ip: ip.to_string(),
            port,
            incoming: Arc::new(Queue::new()),
            outgoing: Arc::new(Queue::new()),
        })
    }

    pub async fn run(&mut self) {
        let addr = format!("{}:{}", self.ip, self.port);
        let socket = UdpSocket::bind(format!("{}:0", self.ip)).await.unwrap();
        if socket.connect(&addr).await.is_err() {
            panic!("Failed to connect to server");
        }

        println!("Server connected - {}", addr);
        self.outgoing.push("Hello, Server!".to_string());

        let incoming = Arc::clone(&self.incoming);
        let outgoing = Arc::clone(&self.outgoing);

        {
            let mut buf = [0; 1024];
            loop {
                let len = socket.recv(&mut buf).await.unwrap();

                let msg = String::from_utf8_lossy(&buf[..len]);
                println!("Received: {}", msg);
                incoming.push(msg.to_string());

                while let Some(s) = outgoing.pop() {
                    println!("Sending: {}", s);
                    socket.send(s.as_bytes()).await.unwrap();
                }
            }
        }
    }

    pub fn send(&mut self, msg: &str) {
        self.outgoing.push(msg.to_string());
    }

    pub fn receive(&mut self) -> Option<String> {
        self.incoming.pop()
    }
}
