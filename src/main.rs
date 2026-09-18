use std::{process::{ExitCode}};

use simple_web_healthcheck::test_connection;

fn main() -> Result<(), ExitCode> {
    test_connection()?;
    Ok(())
}
