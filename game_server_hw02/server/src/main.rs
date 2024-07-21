use server::server;


#[tokio::main]
async fn main() {
    let ip = "127.0.0.1";

    let tcp_server_handle = tokio::spawn(server::tcp_server::run_server(ip, 8080));
    let udp_server_handle = tokio::spawn(server::udp_server::run_server(ip, 7878));

    let (_, _) = tokio::join!(tcp_server_handle, udp_server_handle);
}
