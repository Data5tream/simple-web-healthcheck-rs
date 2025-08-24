use http_body_util::Empty;
use hyper::{Request, body::Bytes};
use hyper_util::rt::TokioIo;
use std::{env, process::exit, time::Duration};
use tokio::{net::TcpStream, time::timeout};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let timeout_duration = Duration::from_secs(5);

    // Parse our URL...
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Missing URL\nUsage: simple-web-healthcheck <url>");
        exit(1);
    }
    let url = &args[1].parse::<hyper::Uri>().unwrap_or_else(|_| {
        eprintln!("Invalid URL: {}", &args[1]);
        exit(2)
    });

    // Get the host and the port
    let host = url.host().expect("uri has no host");
    let port = url.port_u16().unwrap_or(80);

    let address = format!("{host}:{port}");
    println!("Connecting to {address}");

    // Open a TCP connection to the remote host
    let stream = match timeout(timeout_duration, TcpStream::connect(address)).await {
        Ok(stream_result) => match stream_result {
            Ok(stream) => stream,
            Err(_) => exit(1),
        },
        Err(_) => exit(1),
    };

    // Create the Hyper client
    let io = TokioIo::new(stream);
    let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;
    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            eprintln!("Connection failed: {err:?}");
        }
    });

    // Create an HTTP request with an empty body and a HOST header
    let authority = url.authority().unwrap().clone();
    let req = Request::builder()
        .uri(url)
        .header(hyper::header::HOST, authority.as_str())
        .body(Empty::<Bytes>::new())?;

    // Perform the GET request
    let status = match timeout(timeout_duration, sender.send_request(req)).await {
        Ok(result) => match result {
            Ok(result) => result.status(),
            Err(_) => exit(1),
        },
        Err(_) => exit(1),
    };

    if status.is_client_error() || status.is_server_error() {
        exit(4);
    }

    Ok(())
}
