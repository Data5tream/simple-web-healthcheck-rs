use std::{env, process::exit};

use reqwest::blocking::Client;

fn create_client() -> Client {
    Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap()
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Missing URL\nUsage: simple-web-healthcheck <url>");
        exit(1);
    }

    let path = &args[1];

    let client = create_client();
    let response = client.get(path).send().unwrap_or_else(|_| exit(1));

    let status = response.status();

    if status.is_client_error() || status.is_server_error() {
        exit(1);
    }
}
