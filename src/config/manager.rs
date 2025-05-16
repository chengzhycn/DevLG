use crate::models::session::{Session, Template};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub sessions: Vec<Session>,
    pub templates: Vec<Template>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            sessions: Vec::new(),
            templates: Vec::new(),
        }
    }

    pub fn load() -> Result<Self> {
        let config_path = Self::get_config_path()?;
        if !config_path.exists() {
            return Ok(Config::new());
        }

        let content = fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read config file at {:?}", config_path))?;

        let config: Config =
            toml::from_str(&content).with_context(|| "Failed to parse config file")?;

        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::get_config_path()?;
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory at {:?}", parent))?;
        }

        let content = toml::to_string_pretty(self).with_context(|| "Failed to serialize config")?;

        fs::write(&config_path, content)
            .with_context(|| format!("Failed to write config file at {:?}", config_path))?;

        Ok(())
    }

    pub fn add_session(&mut self, session: Session) -> Result<()> {
        if self.sessions.iter().any(|s| s.name == session.name) {
            anyhow::bail!("Session with name '{}' already exists", session.name);
        }
        self.sessions.push(session);
        self.save()
    }

    pub fn remove_session(&mut self, name: &str) -> Result<()> {
        let initial_len = self.sessions.len();
        self.sessions.retain(|s| s.name != name);
        if self.sessions.len() == initial_len {
            anyhow::bail!("Session '{}' not found", name);
        }
        self.save()
    }

    pub fn get_session(&self, name: &str) -> Option<&Session> {
        self.sessions.iter().find(|s| s.name == name)
    }

    pub fn search_sessions(&self, query: &str, tags: &str) -> Vec<&Session> {
        self.sessions
            .iter()
            .filter(|s| s.name.contains(query) || s.host.contains(query))
            .filter(|s| tags.is_empty() || s.tags.contains(tags))
            .collect()
    }

    pub fn update_session(&mut self, session: Session) -> Result<()> {
        if let Some(idx) = self.sessions.iter().position(|s| s.name == session.name) {
            self.sessions[idx] = session;
            self.save()
        } else {
            anyhow::bail!("Session '{}' not found", session.name)
        }
    }

    pub fn add_template(&mut self, template: Template) -> Result<()> {
        if self.templates.iter().any(|t| t.name == template.name) {
            anyhow::bail!("Template with name '{}' already exists", template.name);
        }

        if self.get_session(&template.session).is_none() {
            anyhow::bail!("Session '{}' not found", template.session);
        }

        self.templates.push(template);
        self.save()
    }

    pub fn remove_template(&mut self, name: &str) -> Result<()> {
        let initial_len = self.templates.len();
        self.templates.retain(|t| t.name != name);
        if self.templates.len() == initial_len {
            anyhow::bail!("Template '{}' not found", name);
        }
        self.save()
    }

    #[allow(dead_code)]
    pub fn get_template(&self, name: &str) -> Option<&Template> {
        self.templates.iter().find(|t| t.name == name)
    }

    #[allow(dead_code)]
    pub fn list_templates(&self) -> &[Template] {
        &self.templates
    }

    fn get_config_path() -> Result<PathBuf> {
        let home = dirs::home_dir().context("Failed to get home directory")?;
        Ok(home.join(".config").join("devlg.toml"))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_config_operations() -> Result<()> {
        let temp_dir = tempdir()?;
        let config_path = temp_dir.path().join("devlg.toml");

        // Test new config
        let mut config = Config::new();
        assert!(config.sessions.is_empty());

        // Test adding session
        let session = Session::new(
            "test".to_string(),
            "example.com".to_string(),
            "user".to_string(),
            22,
            crate::models::session::AuthType::Key,
            Some(PathBuf::from("~/.ssh/id_rsa")),
            None,
            Some(HashSet::from(["production".to_string(), "web".to_string()])),
        );
        config.add_session(session.clone())?;
        assert_eq!(config.sessions.len(), 1);

        // Test saving config
        std::fs::write(&config_path, toml::to_string(&config)?)?;

        // Test loading config
        let loaded_config: Config = toml::from_str(&std::fs::read_to_string(&config_path)?)?;
        assert_eq!(loaded_config.sessions.len(), 1);

        // Test getting session
        let found_session = config.get_session("test").unwrap();
        assert_eq!(found_session.name, "test");
        assert_eq!(
            found_session.tags,
            HashSet::from(["production".to_string(), "web".to_string()])
        );

        // Test removing session
        config.remove_session("test")?;
        assert!(config.sessions.is_empty());

        Ok(())
    }
}
