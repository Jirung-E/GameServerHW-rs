use server::server;
use std::env;
use local_ip_address::local_ip;


#[tokio::main]
async fn main() {
    let help_message = "Usage: .exe <mode>:<port>";
    let mode_list = "mode: \n  - localhost\n  - public: \t0.0.0.0\n";

    let help_message = &format!("{}\n{}", help_message, mode_list);

    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("{}", help_message);
        return;
    }

    let addr = &args[1];
    let mut s = addr.split(":");
    let ip = s.next()
        .expect(help_message);
    let port = s.next()
        .expect(help_message)
        .parse::<u16>().unwrap();

    let ip = match ip {
        "localhost" => ip,
        "public" => &local_ip().unwrap().to_string(),
        _ => {
            eprintln!("{}", help_message);
            return;
        }
    };

    // let ip = "127.0.0.1";
    // let port = 8080;

    server::tcp_server::run_server(ip, port).await;

    // let tcp_server_handle = tokio::spawn(server::tcp_server::run_server(ip, 8080));
    // let udp_server_handle = tokio::spawn(server::udp_server::run_server(ip, 7878));

    // let (_, _) = tokio::join!(tcp_server_handle, udp_server_handle);
}
