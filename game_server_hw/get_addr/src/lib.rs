use std::env;
use local_ip_address::local_ip;


pub fn get_addr() -> Result<(String, u16), String> {
    let help_message = "Usage: .exe <mode(or ip)>:<port>";
    let public_ip = local_ip().unwrap().to_string();
    let mode_list = format!("mode: \n  - localhost\n  - public: \t{}\n", public_ip);

    let help_message = format!("{}\n{}", help_message, mode_list);

    let args: Vec<String> = env::args().collect();
    if args.len() > 2 {
        return Err(help_message);
    }

    let addr = if args.len() == 1 {
        "public:8080"
    }
    else {
        &args[1]
    };

    let mut s = addr.split(":");
    let ip = s.next()
        .expect(&help_message);
    let port = s.next()
        .expect(&help_message)
        .parse::<u16>().unwrap();

    let ip = match ip {
        "public" => &public_ip,
        _ => ip
    };

    Ok((ip.to_string(), port))
}