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

    server::tcp_server::run_server(&ip, port).await;
}
