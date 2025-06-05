use anyhow::Result;
use std::env;

pub fn handle_version() -> Result<()> {
    println!("devlg version {}", env!("CARGO_PKG_VERSION"));
    Ok(())
}
