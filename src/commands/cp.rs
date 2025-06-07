use anyhow::Result;
use std::path::PathBuf;

use crate::config::manager::ConfigManager;
use crate::utils::scp;

pub fn handle_cp(
    paths: Vec<PathBuf>,
    src: Option<String>,
    dst: Option<String>,
    recursive: bool,
) -> Result<()> {
    let mut manager = ConfigManager::new(None);
    manager.load()?;

    let src_session = src
        .as_ref()
        .and_then(|name| manager.config.get_session(name));

    let dst_session = dst
        .as_ref()
        .and_then(|name| manager.config.get_session(name));

    if paths.len() < 2 {
        anyhow::bail!("At least two paths are required");
    }

    let src_path = paths[0..paths.len() - 1]
        .iter()
        .map(|p| p.as_path())
        .collect();
    let dst_path = paths[paths.len() - 1].as_path();

    scp::copy_file(src_session, dst_session, src_path, dst_path, recursive)
}
