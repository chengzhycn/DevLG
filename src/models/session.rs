use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    fmt::{self, Display},
    path::PathBuf,
    str::FromStr,
};

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
    #[serde(default)]
    pub tags: HashSet<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Template {
    pub name: String,
    pub session: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum AuthType {
    #[serde(rename = "key")]
    Key,
    #[serde(rename = "password")]
    Password,
}

impl FromStr for AuthType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "key" => AuthType::Key,
            "password" => AuthType::Password,
            _ => bail!("Invalid auth type: {}", s),
        })
    }
}

impl From<AuthType> for String {
    fn from(auth_type: AuthType) -> Self {
        match auth_type {
            AuthType::Key => "key".to_string(),
            AuthType::Password => "password".to_string(),
        }
    }
}

impl Display for AuthType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", Into::<String>::into(*self))
    }
}

#[derive(Default)]
pub struct SessionBuilder {
    name: Option<String>,
    host: Option<String>,
    user: Option<String>,
    port: Option<u16>,
    auth_type: Option<AuthType>,
    private_key_path: Option<PathBuf>,
    password: Option<String>,
    tags: Option<HashSet<String>>,
}

impl SessionBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn host(mut self, host: String) -> Self {
        self.host = Some(host);
        self
    }

    pub fn user(mut self, user: String) -> Self {
        self.user = Some(user);
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    pub fn auth_type(mut self, auth_type: AuthType) -> Self {
        self.auth_type = Some(auth_type);
        self
    }

    pub fn private_key_path(mut self, path: Option<PathBuf>) -> Self {
        self.private_key_path = path;
        self
    }

    pub fn password(mut self, password: Option<String>) -> Self {
        self.password = password;
        self
    }

    pub fn tags(mut self, tags: Option<HashSet<String>>) -> Self {
        self.tags = tags;
        self
    }

    pub fn build(self) -> Result<Session> {
        let session = Session {
            name: self
                .name
                .ok_or_else(|| anyhow::anyhow!("Session name is required"))?,
            host: self
                .host
                .ok_or_else(|| anyhow::anyhow!("Host is required"))?,
            user: self
                .user
                .ok_or_else(|| anyhow::anyhow!("User is required"))?,
            port: self.port.unwrap_or(22),
            auth_type: self
                .auth_type
                .ok_or_else(|| anyhow::anyhow!("Auth type is required"))?,
            private_key_path: self.private_key_path,
            password: self.password,
            tags: self.tags.unwrap_or_default(),
        };

        session.validate()?;
        Ok(session)
    }
}

impl Session {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: String,
        host: String,
        user: String,
        port: u16,
        auth_type: AuthType,
        private_key_path: Option<PathBuf>,
        password: Option<String>,
        tags: Option<HashSet<String>>,
    ) -> Self {
        SessionBuilder::new()
            .name(name)
            .host(host)
            .user(user)
            .port(port)
            .auth_type(auth_type)
            .private_key_path(private_key_path)
            .password(password)
            .tags(tags)
            .build()
            .expect("Failed to build session")
    }

    pub fn empty_template() -> Self {
        SessionBuilder::new()
            .name("".to_string())
            .host("".to_string())
            .user("".to_string())
            .port(22)
            .auth_type(AuthType::Key)
            .private_key_path(None)
            .password(None)
            .tags(None)
            .build()
            .expect("Failed to build empty template")
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
        let valid_session = SessionBuilder::new()
            .name("test".to_string())
            .host("example.com".to_string())
            .user("user".to_string())
            .port(22)
            .auth_type(AuthType::Key)
            .private_key_path(Some(PathBuf::from("~/.ssh/id_rsa")))
            .password(None)
            .tags(Some(HashSet::from([
                "production".to_string(),
                "web".to_string(),
            ])))
            .build()
            .unwrap();
        assert!(valid_session.validate().is_ok());
    }
}
