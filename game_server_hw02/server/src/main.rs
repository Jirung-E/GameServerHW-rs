use server::server;
use get_addr::get_addr;


#[tokio::main]
async fn main() {
    let (ip, port) = match get_addr() {
        Ok((ip, port)) => (ip, port),
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };

    // let ip = "127.0.0.1";
    // let port = 8080;

    server::tcp_server::run_server(&ip, port).await;

    // let tcp_server_handle = tokio::spawn(server::tcp_server::run_server(ip, 8080));
    // let udp_server_handle = tokio::spawn(server::udp_server::run_server(ip, 7878));

    // let (_, _) = tokio::join!(tcp_server_handle, udp_server_handle);
}
