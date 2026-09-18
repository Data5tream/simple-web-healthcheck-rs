use std::{
    error::Error,
    fmt,
    net::{Ipv6Addr, SocketAddr},
    str::FromStr,
};

pub struct HealthcheckAddr {
    pub socket: SocketAddr,
    pub path: String,
}

#[derive(Debug)]
pub enum UrlError {
    MissingArgument,
    UnsupportedSchema,
    HttpsNotSupported,
    InvalidSocketAddr,
}

impl fmt::Display for UrlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UrlError::MissingArgument => write!(f, "missing url argument"),
            UrlError::UnsupportedSchema => write!(f, "unsupported schema"),
            UrlError::HttpsNotSupported => write!(f, "HTTPS is not supported"),
            UrlError::InvalidSocketAddr => write!(f, "invalid socket address"),
        }
    }
}

impl Error for UrlError {}

pub fn get_connection_url(args: &[String]) -> Result<HealthcheckAddr, UrlError> {
    if args.len() != 2 {
        eprintln!("Invalid arguments\nUsage: simple-web-healthcheck <URL>");
        return Err(UrlError::MissingArgument);
    }

    let raw_url = &args[1];

    if raw_url.len() > 8 && &raw_url[..8] == "https://" {
        return Err(UrlError::HttpsNotSupported);
    }

    let mut addr_start = raw_url.find("://").unwrap_or_default();
    if addr_start != 0 {
        if raw_url.len() > 7 && &raw_url[..7] != "http://" {
            return Err(UrlError::UnsupportedSchema);
        }
        // make sure we start after the `://` sequence
        addr_start += 3;
    }

    let without_schema = &raw_url[addr_start..];

    // offset the addr end by the start of the actual host part
    let addr_end = without_schema.find('/').unwrap_or(without_schema.len());

    let mut host_string = String::from(&without_schema[..addr_end]);
    // this breaks on IPv6
    if let Some(i) = host_string.rfind(':') {
        if host_string.matches(':').count() > 1 && Ipv6Addr::from_str(&host_string).is_ok() {
            // we got a full IPv6 without port, so add default port
            host_string.push_str(":80");
        } else {
            // not a valid raw IPv6, if last segment isn't a port, add default port
            let test_string = &host_string[i + 1..];
            if test_string.parse::<usize>().is_err() {
                host_string.push_str(":80");
            }
        }
    } else {
        // no port found, add default port
        host_string.push_str(":80");
    }

    let path = if without_schema.len() > addr_end {
        String::from(&without_schema[addr_end..])
    } else {
        String::from("/")
    };

    println!("{host_string}{path}");

    let Ok(socket) = SocketAddr::from_str(&host_string) else {
        println!("invalid host string: {host_string}");
        return Err(UrlError::InvalidSocketAddr);
    };

    Ok(HealthcheckAddr { socket, path })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

    #[test]
    fn missing_argument_returns_correct_error() {
        let args = vec![String::from("cmd")];
        let result = get_connection_url(&args);
        assert!(result.is_err_and(|e| matches!(e, UrlError::MissingArgument)));
    }

    #[test]
    fn https_is_not_supported() {
        let args = vec![String::from("cmd"), String::from("https://test")];
        let result = get_connection_url(&args);
        assert!(result.is_err_and(|e| matches!(e, UrlError::HttpsNotSupported)));
    }

    #[test]
    fn http_is_supported() {
        let args = vec![String::from("cmd"), String::from("http://127.0.0.1")];
        let result = get_connection_url(&args);
        assert!(result.is_ok());
    }

    #[test]
    fn schemas_besides_http_or_https_throw_generic_schema_error() {
        let args = vec![String::from("cmd"), String::from("ftp://test")];
        let result = get_connection_url(&args);
        assert!(result.is_err_and(|e| matches!(e, UrlError::UnsupportedSchema)));
    }

    #[test]
    fn ipv4_with_port_and_without_schema_and_path_resolves() {
        let args = vec![String::from("cmd"), String::from("127.0.0.1:3000")];
        let result = get_connection_url(&args);

        assert!(result.is_ok());
        let socket = result.unwrap();
        assert_eq!(
            socket.socket,
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 3000)
        );
        assert_eq!(socket.path, String::from("/"));
    }

    #[test]
    fn ipv6_with_port_and_without_schema_and_path_resolves() {
        let args = vec![String::from("cmd"), String::from("[::1]:3000")];
        let result = get_connection_url(&args);

        assert!(result.is_ok());
        let socket = result.unwrap();
        assert_eq!(
            socket.socket,
            SocketAddr::new(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)), 3000)
        );
        assert_eq!(socket.path, String::from("/"));
    }

    #[test]
    fn ipv4_without_port_schema_and_path_resolves() {
        let args = vec![String::from("cmd"), String::from("127.0.0.1")];
        let result = get_connection_url(&args);

        assert!(result.is_ok());
        let socket = result.unwrap();
        assert_eq!(
            socket.socket,
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 80)
        );
        assert_eq!(socket.path, String::from("/"));
    }

    #[test]
    fn ipv6_without_port_and_schema_and_path_resolves() {
        let args = vec![String::from("cmd"), String::from("[::1]")];
        let result = get_connection_url(&args);

        assert!(result.is_ok());
        let socket = result.unwrap();
        assert_eq!(
            socket.socket,
            SocketAddr::new(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)), 80)
        );
        assert_eq!(socket.path, String::from("/"));
    }

    #[test]
    fn ipv4_with_port_and_path_without_schema_resolves() {
        let args = vec![String::from("cmd"), String::from("127.0.0.1:3000/test")];
        let result = get_connection_url(&args);

        assert!(result.is_ok());
        let socket = result.unwrap();
        assert_eq!(
            socket.socket,
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 3000)
        );
        assert_eq!(socket.path, String::from("/test"));
    }

    #[test]
    fn ipv6_with_port_and_path_without_schema_resolves() {
        let args = vec![String::from("cmd"), String::from("[::1]:3000/test")];
        let result = get_connection_url(&args);

        assert!(result.is_ok());
        let socket = result.unwrap();
        assert_eq!(
            socket.socket,
            SocketAddr::new(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)), 3000)
        );
        assert_eq!(socket.path, String::from("/test"));
    }

    #[test]
    fn ipv4_with_path_and_without_port_schema_resolves() {
        let args = vec![String::from("cmd"), String::from("127.0.0.1/test")];
        let result = get_connection_url(&args);

        assert!(result.is_ok());
        let socket = result.unwrap();
        assert_eq!(
            socket.socket,
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 80)
        );
        assert_eq!(socket.path, String::from("/test"));
    }

    #[test]
    fn ipv6_with_path_and_without_port_and_schema_resolves() {
        let args = vec![String::from("cmd"), String::from("[::1]/test")];
        let result = get_connection_url(&args);

        assert!(result.is_ok());
        let socket = result.unwrap();
        assert_eq!(
            socket.socket,
            SocketAddr::new(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)), 80)
        );
        assert_eq!(socket.path, String::from("/test"));
    }

    #[test]
    fn ipv4_with_port_and_schema_without_path_resolves() {
        let args = vec![String::from("cmd"), String::from("http://127.0.0.1:3000")];
        let result = get_connection_url(&args);

        assert!(result.is_ok());
        let socket = result.unwrap();
        assert_eq!(
            socket.socket,
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 3000)
        );
        assert_eq!(socket.path, String::from("/"));
    }

    #[test]
    fn ipv6_with_port_and_with_schema_and_without_path_resolves() {
        let args = vec![String::from("cmd"), String::from("http://[::1]:3000")];
        let result = get_connection_url(&args);

        assert!(result.is_ok());
        let socket = result.unwrap();
        assert_eq!(
            socket.socket,
            SocketAddr::new(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)), 3000)
        );
        assert_eq!(socket.path, String::from("/"));
    }

    #[test]
    fn ipv4_with_schema_and_without_port_and_path_resolves() {
        let args = vec![String::from("cmd"), String::from("http://127.0.0.1")];
        let result = get_connection_url(&args);

        assert!(result.is_ok());
        let socket = result.unwrap();
        assert_eq!(
            socket.socket,
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 80)
        );
        assert_eq!(socket.path, String::from("/"));
    }

    #[test]
    fn ipv6_with_schema_and_without_port_and_path_resolves() {
        let args = vec![String::from("cmd"), String::from("http://[::1]")];
        let result = get_connection_url(&args);

        assert!(result.is_ok());
        let socket = result.unwrap();
        assert_eq!(
            socket.socket,
            SocketAddr::new(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)), 80)
        );
        assert_eq!(socket.path, String::from("/"));
    }

    #[test]
    fn ipv4_with_port_and_schema_and_with_path_resolves() {
        let args = vec![
            String::from("cmd"),
            String::from("http://127.0.0.1:3000/test"),
        ];
        let result = get_connection_url(&args);

        assert!(result.is_ok());
        let socket = result.unwrap();
        assert_eq!(
            socket.socket,
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 3000)
        );
        assert_eq!(socket.path, String::from("/test"));
    }

    #[test]
    fn ipv6_with_port_schema_and_path_resolves() {
        let args = vec![String::from("cmd"), String::from("http://[::1]:3000/test")];
        let result = get_connection_url(&args);

        assert!(result.is_ok());
        let socket = result.unwrap();
        assert_eq!(
            socket.socket,
            SocketAddr::new(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)), 3000)
        );
        assert_eq!(socket.path, String::from("/test"));
    }

    #[test]
    fn ipv4_with_path_and_schema_and_without_port_resolves() {
        let args = vec![String::from("cmd"), String::from("http://127.0.0.1/test")];
        let result = get_connection_url(&args);

        assert!(result.is_ok());
        let socket = result.unwrap();
        assert_eq!(
            socket.socket,
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 80)
        );
        assert_eq!(socket.path, String::from("/test"));
    }

    #[test]
    fn ipv6_with_path_and_schema_and_without_port_resolves() {
        let args = vec![String::from("cmd"), String::from("http://[::1]/test")];
        let result = get_connection_url(&args);

        assert!(result.is_ok());
        let socket = result.unwrap();
        assert_eq!(
            socket.socket,
            SocketAddr::new(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)), 80)
        );
        assert_eq!(socket.path, String::from("/test"));
    }
}
