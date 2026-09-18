use std::{
    fmt,
    io::{self, Read, Write},
    net::TcpStream,
};

use crate::{address::HealthcheckAddr, connection::TestError::HttpError};

#[derive(Debug)]
pub enum TestError {
    ConnectionFailure(io::Error),
    RequestSendingError,
    ResponseReadingError(io::Error),
    MalformedHttp,
    HttpError(usize),
}

impl fmt::Display for TestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TestError::ConnectionFailure(err) => write!(f, "TCP connection failed: {err}"),
            TestError::RequestSendingError => write!(f, "failed to send HTTP request"),
            TestError::ResponseReadingError(err) => {
                write!(f, "failed to read HTTP response: {err}")
            }
            TestError::MalformedHttp => write!(f, "response is not valid HTTP"),
            TestError::HttpError(status) => write!(f, "response has HTTP status {status}"),
        }
    }
}

fn parse_response(response: &str) -> Result<(), TestError> {
    if !response.starts_with("HTTP/1.1") {
        return Err(TestError::MalformedHttp);
    }

    let Some(first_newline) = response.find('\n') else {
        return Err(TestError::MalformedHttp);
    };

    let first_line = &response[..first_newline];
    let parts: Vec<&str> = first_line.split(' ').collect();
    let Ok(response_code) = parts[1].parse::<usize>() else {
        return Err(TestError::MalformedHttp);
    };

    if (400..600).contains(&response_code) {
        return Err(HttpError(response_code));
    }

    Ok(())
}

pub fn connect(connection_details: &HealthcheckAddr) -> Result<(), TestError> {
    let mut stream = match TcpStream::connect(connection_details.socket) {
        Ok(stream) => stream,
        Err(err) => {
            return Err(TestError::ConnectionFailure(err));
        }
    };

    let request = format!(
        "GET {} HTTP/1.1\n\
        Host: {}\n\
        User-agent: simple-web-healthcheck-rs\n\
        Connection: close\n\n",
        connection_details.path,
        connection_details.socket.ip()
    );

    if let Err(err) = stream.write_all(request.as_bytes()) {
        eprintln!("{err}");
        return Err(TestError::RequestSendingError);
    }

    let mut response = String::new();
    if let Err(err) = stream.read_to_string(&mut response) {
        return Err(TestError::ResponseReadingError(err));
    }

    parse_response(&response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_http_response_produces_error() {
        let raw = String::from("HASD ASD ASD as d");
        let parsed = parse_response(&raw);

        assert!(parsed.is_err_and(|e| matches!(e, TestError::MalformedHttp)));
    }

    #[test]
    fn partial_http_response_produces_error() {
        let raw = String::from("HTTP/1.1 200 OK");
        let parsed = parse_response(&raw);

        assert!(parsed.is_err_and(|e| matches!(e, TestError::MalformedHttp)));
    }

    #[test]
    fn http_response_with_invalid_status_produces_error() {
        let raw = String::from("HTTP/1.1 200A OK");
        let parsed = parse_response(&raw);

        assert!(parsed.is_err_and(|e| matches!(e, TestError::MalformedHttp)));
    }

    #[test]
    fn http_response_with_client_error_status_produces_error() {
        let raw = String::from("HTTP/1.1 404 NOT_FOUND\n");
        let parsed = parse_response(&raw);

        assert!(parsed.is_err());
        let error = parsed.unwrap_err();
        match error {
            TestError::HttpError(status) => assert_eq!(status, 404),
            other => panic!("wrong error code: {other}"),
        }
    }

    #[test]
    fn http_response_with_server_error_status_produces_error() {
        let raw = String::from("HTTP/1.1 520 X\n");
        let parsed = parse_response(&raw);

        assert!(parsed.is_err());
        let error = parsed.unwrap_err();
        match error {
            TestError::HttpError(status) => assert_eq!(status, 520),
            other => panic!("wrong error code: {other}"),
        }
    }

    #[test]
    fn http_response_with_success_status_code_is_valid() {
        let raw = String::from("HTTP/1.1 200 SUCESS\n");
        let parsed = parse_response(&raw);

        assert!(parsed.is_ok());
    }

    #[test]
    fn http_response_with_redirect_status_code_is_valid() {
        let raw = String::from("HTTP/1.1 301 REDIRECT\n");
        let parsed = parse_response(&raw);

        assert!(parsed.is_ok());
    }

    #[test]
    fn http_response_with_other_status_code_is_valid() {
        let raw = String::from("HTTP/1.1 101 B\n");
        let parsed = parse_response(&raw);

        assert!(parsed.is_ok());
    }
}
