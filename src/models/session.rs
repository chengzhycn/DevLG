use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Session {
    pub name: String,
    pub host: String,
    pub user: String,
    pub port: u16,
    pub auth_type: AuthType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_key_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum AuthType {
    #[serde(rename = "key")]
    Key,
    #[serde(rename = "password")]
    Password,
}

impl Session {
    pub fn new(
        name: String,
        host: String,
        user: String,
        port: u16,
        auth_type: AuthType,
        private_key_path: Option<PathBuf>,
        password: Option<String>,
    ) -> Self {
        Session {
            name,
            host,
            user,
            port,
            auth_type,
            private_key_path,
            password,
        }
    }

    pub fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            bail!("Session name cannot be empty");
        }
        if self.host.is_empty() {
            bail!("Host cannot be empty");
        }
        if self.user.is_empty() {
            bail!("User cannot be empty");
        }
        if self.port == 0 {
            bail!("Port cannot be 0");
        }

        match self.auth_type {
            AuthType::Key => {
                if self.private_key_path.is_none() {
                    bail!("Private key path is required for key authentication");
                }
            }
            AuthType::Password => {
                if self.password.is_none() {
                    bail!("Password is required for password authentication");
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_validation() {
        let valid_session = Session::new(
            "test".to_string(),
            "example.com".to_string(),
            "user".to_string(),
            22,
            AuthType::Key,
            Some(PathBuf::from("~/.ssh/id_rsa")),
            None,
        );
        assert!(valid_session.validate().is_ok());

        let invalid_session = Session::new(
            "".to_string(),
            "example.com".to_string(),
            "user".to_string(),
            22,
            AuthType::Key,
            Some(PathBuf::from("~/.ssh/id_rsa")),
            None,
        );
        assert!(invalid_session.validate().is_err());
    }
}
