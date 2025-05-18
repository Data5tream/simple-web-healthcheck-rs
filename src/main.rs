fn main() {
    reqwest::blocking::get("http://127.0.0.1:8080/ping").unwrap_or_else(|_| {
        std::process::exit(1);
    });
}
