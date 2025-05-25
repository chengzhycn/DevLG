use crate::models::session::Session;
use anyhow::{Context, Result};
use std::{path::Path, process::Command};

pub fn copy_file(
    src_session: Option<&Session>,
    dst_session: Option<&Session>,
    src_path: &Path,
    dst_path: &Path,
) -> Result<()> {
    let mut s_bits = 0;
    let src_uri = if let Some(session) = src_session {
        s_bits |= 1;
        generate_scp_uri(session, src_path)
    } else {
        src_path.to_string_lossy().to_string()
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

    let mut cmd = match sess.auth_type {
        crate::models::session::AuthType::Password => {
            let mut cmd = Command::new("sshpass");
            cmd.arg("-p")
                .arg(sess.password.as_ref().context("Password not found")?);
            cmd.arg("scp");
            cmd
        }
        crate::models::session::AuthType::Key => {
            let mut cmd = Command::new("scp");
            cmd.arg("-i").arg(
                sess.private_key_path
                    .as_ref()
                    .context("Private key not found")?,
            );
            cmd
        }
    };

    cmd.arg(src_uri).arg(dst_uri);

    let status = cmd.status().context("Failed to execute SCP command")?;
    if !status.success() {
        anyhow::bail!("SCP command failed with exit code: {}", status);
    }

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
