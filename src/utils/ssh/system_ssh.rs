use crate::models::session::Session;
use anyhow::Context;
use std::process::Command;

/// Implementation using the system's SSH client
pub struct SystemSshConnector;

impl super::SshConnector for SystemSshConnector {
    fn connect(&self, session: &Session) -> anyhow::Result<()> {
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
}
