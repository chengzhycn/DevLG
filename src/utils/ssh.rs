use crate::models::session::Session;

mod ssh2_connector;
mod system_ssh;

pub use ssh2_connector::Ssh2Connector;
pub use system_ssh::SystemSshConnector;

/// Trait for SSH connection implementations
pub trait SshConnector {
    /// Establishes an SSH connection to the remote server.
    ///
    /// # Arguments
    ///
    /// * `session` - The SSH session configuration
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the connection was successful
    /// * `Err(_)` - If the connection failed
    fn connect(&self, session: &Session) -> anyhow::Result<()>;
}
