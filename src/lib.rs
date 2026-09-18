use std::{
    env,
    process::{ExitCode, exit},
};

mod address;
mod connection;

use address::get_connection_url;

use crate::connection::{TestError, connect};

pub fn test_connection() -> Result<(), ExitCode> {
    let args: Vec<String> = env::args().collect();
    let connection_details = match get_connection_url(&args) {
        Ok(details) => details,
        Err(err) => {
            eprintln!("failed to parse URL: {err}");
            exit(2);
        }
    };

    match connect(&connection_details) {
        Ok(()) => Ok(()),
        Err(err) => {
            eprintln!("{err}");
            if matches!(err, TestError::HttpError(_)) {
                exit(4);
            }
            exit(1);
        }
    }
}
