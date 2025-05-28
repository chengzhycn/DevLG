use crate::{
    models::session::Session,
    utils::ssh::{master_ssh_close, master_ssh_create},
};
use anyhow::{Context, Result};
use std::{path::Path, process::Command};

pub fn copy_file(
    src_session: Option<&Session>,
    dst_session: Option<&Session>,
    src_path: Vec<&Path>,
    dst_path: &Path,
    recursive: bool,
) -> Result<()> {
    let mut s_bits = 0;
    let src_uri: Vec<String> = if let Some(session) = src_session {
        s_bits |= 1;
        src_path
            .iter()
            .map(|p| generate_scp_uri(session, p))
            .collect()
    } else {
        src_path
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect()
    };

    let dst_uri = if let Some(session) = dst_session {
        s_bits |= 1 << 1;
        generate_scp_uri(session, dst_path)
    } else {
        dst_path.to_string_lossy().to_string()
    };

    if s_bits == 3 {
        anyhow::bail!("Both source and destination remote paths are not supported now");
    }

    if s_bits == 0 {
        anyhow::bail!("No session is specified");
    }

    let sess = if s_bits == 1 {
        src_session.unwrap()
    } else {
        dst_session.unwrap()
    };

    // first create a master ssh connection
    let control_path = master_ssh_create(sess).context("Failed to create master SSH connection")?;

    let mut cmd = Command::new("scp");
    cmd.arg("-o")
        .arg(format!("ControlPath={}", control_path.display()));

    if recursive {
        cmd.arg("-r");
    }

    for src in src_uri.clone() {
        cmd.arg(src);
    }
    cmd.arg(dst_uri.clone());

    let status = cmd.status().context("Failed to execute SCP command")?;
    if !status.success() {
        anyhow::bail!("SCP command failed with exit code: {}", status);
    }

    master_ssh_close(sess).context("Failed to close master SSH connection")?;

    println!(
        "copy file from {} to {} success.",
        src_uri.join(" "),
        dst_uri
    );

    Ok(())
}

fn generate_scp_uri(session: &Session, path: &Path) -> String {
    let mut uri = String::from("scp://");
    uri.push_str(&session.user);
    uri.push('@');
    uri.push_str(&session.host);
    uri.push(':');
    uri.push_str(&session.port.to_string());
    uri.push('/');
    uri.push_str(&path.to_string_lossy());
    uri
}
