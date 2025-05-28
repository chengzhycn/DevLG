use crate::models::session::Session;
use anyhow::{Context, Ok, Result};
use std::{path::PathBuf, process::Command};

/// Establishes an SSH connection to the remote server using the system's SSH client.
///
/// This function uses the system's SSH client to establish a connection to the remote server.
/// It supports both password and key-based authentication.
///
/// # Arguments
///
/// * `session` - The SSH session configuration
///
/// # Returns
///
/// * `Ok(())` - If the connection was successful
/// * `Err(_)` - If the connection failed
pub fn connect_ssh(session: &Session) -> Result<()> {
    println!(
        "Connecting to {}@{}:{}...",
        session.user, session.host, session.port
    );

    let mut cmd = match session.auth_type {
        crate::models::session::AuthType::Password => {
            // Use sshpass for password authentication
            let mut cmd = Command::new("sshpass");
            cmd.arg("-p")
                .arg(session.password.as_ref().context("Password not found")?);
            cmd.arg("ssh");
            cmd
        }
        crate::models::session::AuthType::Key => {
            // Use regular ssh for key authentication
            Command::new("ssh")
        }
    };

    // Add port
    cmd.arg("-p").arg(session.port.to_string());

    // Add user
    cmd.arg("-l").arg(&session.user);

    // Add option StrictHostKeyChecking=accept-new
    cmd.arg("-o").arg("StrictHostKeyChecking=accept-new");

    // Add identity file if using key authentication
    if let crate::models::session::AuthType::Key = session.auth_type {
        if let Some(key_path) = &session.private_key_path {
            cmd.arg("-i").arg(key_path);
        }
    }

    // Add host
    cmd.arg(&session.host);

    // Execute the SSH command
    let status = cmd.status().context("Failed to execute SSH command")?;

    if !status.success() {
        anyhow::bail!("SSH connection failed with exit code: {}", status);
    }

    Ok(())
}

/// Create a master SSH connection to the remote server.
/// ssh parameters:
/// -M: master mode
/// -f: run in background
/// -N: do not execute a remote command
/// -o StrictHostKeyChecking=accept-new: accept new host keys
/// -o ExitOnForwardFailure=yes: exit if forwarding fails
/// -o ControlPath=~/.ssh/<session_name>
pub fn master_ssh_create(session: &Session) -> Result<PathBuf> {
    let mut cmd = match session.auth_type {
        crate::models::session::AuthType::Password => {
            // Use sshpass for password authentication
            let mut cmd = Command::new("sshpass");
            cmd.arg("-p")
                .arg(session.password.as_ref().context("Password not found")?);
            cmd.arg("ssh");
            cmd
        }
        crate::models::session::AuthType::Key => {
            // Use regular ssh for key authentication
            Command::new("ssh")
        }
    };

    cmd.arg("-M")
        .arg("-fN")
        .arg("-o")
        .arg("StrictHostKeyChecking=accept-new")
        .arg("-o")
        .arg("ExitOnForwardFailure=yes")
        .arg("-o")
        .arg(format!("ControlPath=~/.ssh/{}", session.name));

    // Add port
    cmd.arg("-p").arg(session.port.to_string());

    // Add user
    cmd.arg("-l").arg(&session.user);

    // Add identity file if using key authentication
    if let crate::models::session::AuthType::Key = session.auth_type {
        if let Some(key_path) = &session.private_key_path {
            cmd.arg("-i").arg(key_path);
        }
    }

    // Add host
    cmd.arg(&session.host);

    // Execute the SSH command
    let status = cmd.status().context("Failed to execute SSH command")?;

    if !status.success() {
        anyhow::bail!("SSH connection failed with exit code: {}", status);
    }

    Ok(PathBuf::from(format!("~/.ssh/{}", session.name)))
}

/// Close the master SSH connection to the remote server.
/// ssh parameters:
/// -O: exit
/// -S: control path
pub fn master_ssh_close(session: &Session) -> Result<()> {
    let control_path = PathBuf::from(format!("~/.ssh/{}", session.name));
    // FIXME: PathBuf.exists() is not working for the socket file, why?
    // if !control_path.exists() {
    //     println!("Control path does not exist: {}", control_path.display());
    //     return Ok(());
    // }

    let mut cmd = Command::new("ssh");
    cmd.arg("-S")
        .arg(control_path.to_string_lossy().to_string())
        .arg("-O")
        .arg("exit")
        .arg(format!(
            "{}@{}:{}",
            session.user, session.host, session.port
        ));

    // TODO: redirect stderr to /dev/null
    let status = cmd.status().context("Failed to execute SSH command")?;

    if !status.success() {
        anyhow::bail!("SSH connection failed with exit code: {}", status);
    }

    Ok(())
}

/// Establishes an SSH connection to the remote server using the ssh2 crate.
///
/// This function uses the ssh2 crate to establish a connection to the remote server.
/// It supports both password and key-based authentication.
///
/// # Arguments
///
/// * `session` - The SSH session configuration
///
/// # Returns
///
/// * `Ok(())` - If the connection was successful
/// * `Err(_)` - If the connection failed
///
/// # Note
///
/// This function is not yet implemented. It will be implemented in a future version.
#[allow(dead_code)]
pub fn connect_ssh2(session: &Session) -> Result<()> {
    // TODO: Implement SSH connection using ssh2 crate
    println!("SSH2 connection not yet implemented. Using system SSH client instead.");
    connect_ssh(session)
}
