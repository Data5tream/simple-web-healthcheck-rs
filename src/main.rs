use std::{env, process::exit};
use ureq::Error;

fn main() {
    // Parse our URL...
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Missing URL\nUsage: simple-web-healthcheck <url>");
        exit(1);
    }
    let url = &args[1].parse::<String>().unwrap_or_else(|_| {
        eprintln!("Invalid URL: {}", &args[1]);
        exit(2)
    });

    println!("Connecting to {url}");

    match ureq::get(url).call() {
        Ok(_) => {
            exit(0);
        }
        Err(Error::StatusCode(code)) => {
            eprintln!("Server answered with {code}");
            exit(4);
        }
        Err(_) => exit(1),
    }
}
