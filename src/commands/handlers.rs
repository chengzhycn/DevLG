use crate::commands::Commands;
use crate::config::manager::Config;
use crate::models::session::{AuthType, Session};
use crate::utils::ssh;
use anyhow::{Context, Result};
use dialoguer::{Input, Select};
use rpassword::read_password;
use std::collections::HashSet;
use std::path::PathBuf;

// Helper function to parse tags from a string
fn parse_tags(tags_str: Option<&String>) -> Vec<String> {
    tags_str
        .map(|s| {
            s.split([',', ';'])
                .map(|tag| tag.trim().to_string())
                .filter(|tag| !tag.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

#[derive(Default)]
struct SessionModification {
    host: Option<String>,
    user: Option<String>,
    port: Option<u16>,
    auth_type: Option<String>,
    key_path: Option<PathBuf>,
    password: Option<String>,
    tags: Option<String>,
}

impl SessionModification {
    fn new() -> Self {
        Self::default()
    }

    fn with_host(mut self, host: Option<String>) -> Self {
        self.host = host;
        self
    }

    fn with_user(mut self, user: Option<String>) -> Self {
        self.user = user;
        self
    }

    fn with_port(mut self, port: Option<u16>) -> Self {
        self.port = port;
        self
    }

    fn with_auth_type(mut self, auth_type: Option<String>) -> Self {
        self.auth_type = auth_type;
        self
    }

    fn with_key_path(mut self, key_path: Option<PathBuf>) -> Self {
        self.key_path = key_path;
        self
    }

    fn with_password(mut self, password: Option<String>) -> Self {
        self.password = password;
        self
    }

    fn with_tags(mut self, tags: Option<String>) -> Self {
        self.tags = tags;
        self
    }
}

#[derive(Default)]
struct SessionAddition {
    name: Option<String>,
    host: Option<String>,
    user: Option<String>,
    port: Option<u16>,
    auth_type: Option<String>,
    key_path: Option<PathBuf>,
    password: Option<String>,
    tags: Option<String>,
}

impl SessionAddition {
    fn new() -> Self {
        Self::default()
    }

    fn with_name(mut self, name: Option<String>) -> Self {
        self.name = name;
        self
    }

    fn with_host(mut self, host: Option<String>) -> Self {
        self.host = host;
        self
    }

    fn with_user(mut self, user: Option<String>) -> Self {
        self.user = user;
        self
    }

    fn with_port(mut self, port: Option<u16>) -> Self {
        self.port = port;
        self
    }

    fn with_auth_type(mut self, auth_type: Option<String>) -> Self {
        self.auth_type = auth_type;
        self
    }

    fn with_key_path(mut self, key_path: Option<PathBuf>) -> Self {
        self.key_path = key_path;
        self
    }

    fn with_password(mut self, password: Option<String>) -> Self {
        self.password = password;
        self
    }

    fn with_tags(mut self, tags: Option<String>) -> Self {
        self.tags = tags;
        self
    }
}

pub async fn handle_command(command: Commands) -> Result<()> {
    match command {
        Commands::List { detailed, tags } => handle_list(detailed, tags),
        Commands::Add {
            name,
            host,
            user,
            port,
            auth_type,
            key_path,
            password,
            tags,
        } => {
            let addition = SessionAddition::new()
                .with_name(name)
                .with_host(host)
                .with_user(user)
                .with_port(port)
                .with_auth_type(auth_type)
                .with_key_path(key_path)
                .with_password(password)
                .with_tags(tags);
            handle_add(addition).await
        }
        Commands::Delete { name } => handle_delete(name),
        Commands::Modify {
            name,
            host,
            user,
            port,
            auth_type,
            key_path,
            password,
            tags,
        } => {
            let modification = SessionModification::new()
                .with_host(host)
                .with_user(user)
                .with_port(port)
                .with_auth_type(auth_type)
                .with_key_path(key_path)
                .with_password(password)
                .with_tags(tags);
            handle_modify(name, modification).await
        }
        Commands::Login { name, tags } => handle_login(name, tags).await,
        Commands::Tag { name, action, tags } => handle_tag(name, action, tags),
    }
}

fn handle_list(detailed: bool, tags_filter: Option<String>) -> Result<()> {
    let config = Config::load()?;
    if config.sessions.is_empty() {
        println!("No SSH sessions found.");
        return Ok(());
    }

    // Filter sessions by tags if specified
    let filtered_sessions: Vec<&Session> = if let Some(tags_str) = tags_filter {
        let filter_tags: HashSet<String> = parse_tags(Some(&tags_str)).into_iter().collect();
        config
            .sessions
            .iter()
            .filter(|session| {
                let session_tags: HashSet<String> = session.tags.iter().cloned().collect();
                !filter_tags.is_disjoint(&session_tags)
            })
            .collect()
    } else {
        config.sessions.iter().collect()
    };

    if filtered_sessions.is_empty() {
        println!("No SSH sessions found matching the specified tags.");
        return Ok(());
    }

    println!("Available SSH sessions:");
    if detailed {
        println!(
            "{:<20} {:<15} {:<10} {:<6} {:<10} {:<20} {:<20}",
            "Name", "Host", "User", "Port", "Auth Type", "Key Path", "Tags"
        );
        println!("{:-<105}", "");

        for session in filtered_sessions.iter() {
            let auth_type = match session.auth_type {
                AuthType::Key => "Key",
                AuthType::Password => "Password",
            };

            let key_path = session
                .private_key_path
                .as_ref()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| "N/A".to_string());

            let tags_str = if session.tags.is_empty() {
                "N/A".to_string()
            } else {
                session.tags.join(", ")
            };

            println!(
                "{:<20} {:<15} {:<10} {:<6} {:<10} {:<20} {:<20}",
                session.name,
                session.host,
                session.user,
                session.port,
                auth_type,
                key_path,
                tags_str
            );
        }
    } else {
        for (i, session) in filtered_sessions.iter().enumerate() {
            let tags_str = if session.tags.is_empty() {
                "".to_string()
            } else {
                format!(" [{}]", session.tags.join(", "))
            };

            println!(
                "{}. {} ({}@{}:{}){}",
                i + 1,
                session.name,
                session.user,
                session.host,
                session.port,
                tags_str
            );
        }
    }
    Ok(())
}

async fn handle_add(addition: SessionAddition) -> Result<()> {
    let mut config = Config::load()?;

    let session = if addition.name.is_some() && addition.host.is_some() && addition.user.is_some() {
        // Command line mode
        let auth_type = match addition
            .auth_type
            .unwrap_or_else(|| "key".to_string())
            .as_str()
        {
            "key" => AuthType::Key,
            "password" => AuthType::Password,
            _ => anyhow::bail!("Invalid authentication type. Use 'key' or 'password'"),
        };

        Session::new(
            addition.name.unwrap(),
            addition.host.unwrap(),
            addition.user.unwrap(),
            addition.port.unwrap_or(22),
            auth_type,
            addition.key_path,
            addition.password,
            Some(parse_tags(addition.tags.as_ref())),
        )
    } else {
        // Interactive mode
        let name: String = Input::new().with_prompt("Session name").interact_text()?;

        let host: String = Input::new().with_prompt("Host").interact_text()?;

        let user: String = Input::new().with_prompt("Username").interact_text()?;

        let port: u16 = Input::new()
            .with_prompt("Port")
            .default(22)
            .interact_text()?;

        let auth_types = vec!["key", "password"];
        let auth_type_idx = Select::new()
            .with_prompt("Authentication type")
            .items(&auth_types)
            .default(0)
            .interact()?;

        let (auth_type, private_key_path, password) = match auth_types[auth_type_idx] {
            "key" => {
                let key_path: String = Input::new()
                    .with_prompt("Private key path")
                    .default("~/.ssh/id_rsa".to_string())
                    .interact_text()?;
                (AuthType::Key, Some(PathBuf::from(key_path)), None)
            }
            "password" => {
                let password = read_password().context("Failed to read password")?;
                (AuthType::Password, None, Some(password))
            }
            _ => unreachable!(),
        };

        let tags_input: String = Input::new()
            .with_prompt("Tags (comma or semicolon separated)")
            .allow_empty(true)
            .interact_text()?;

        let tags = if tags_input.is_empty() {
            None
        } else {
            Some(parse_tags(Some(&tags_input)))
        };

        Session::new(
            name,
            host,
            user,
            port,
            auth_type,
            private_key_path,
            password,
            tags,
        )
    };

    session.validate()?;
    config.add_session(session)?;
    println!("Session added successfully.");
    Ok(())
}

fn handle_delete(name: String) -> Result<()> {
    let mut config = Config::load()?;
    config.remove_session(&name)?;
    println!("Session '{}' deleted successfully.", name);
    Ok(())
}

async fn handle_modify(name: String, modification: SessionModification) -> Result<()> {
    let mut config = Config::load()?;
    let session = config
        .get_session(&name)
        .context("Session not found")?
        .clone();

    let new_session = if modification.host.is_some()
        || modification.user.is_some()
        || modification.port.is_some()
        || modification.auth_type.is_some()
        || modification.key_path.is_some()
        || modification.password.is_some()
        || modification.tags.is_some()
    {
        // Command line mode
        let auth_type = match modification
            .auth_type
            .unwrap_or_else(|| {
                match session.auth_type {
                    AuthType::Key => "key",
                    AuthType::Password => "password",
                }
                .to_string()
            })
            .as_str()
        {
            "key" => AuthType::Key,
            "password" => AuthType::Password,
            _ => anyhow::bail!("Invalid authentication type. Use 'key' or 'password'"),
        };

        Session::new(
            session.name,
            modification.host.unwrap_or(session.host),
            modification.user.unwrap_or(session.user),
            modification.port.unwrap_or(session.port),
            auth_type,
            modification.key_path.or(session.private_key_path),
            modification.password.or(session.password),
            Some(
                modification
                    .tags
                    .map_or_else(|| session.tags.clone(), |s| parse_tags(Some(&s))),
            ),
        )
    } else {
        // Interactive mode
        let host: String = Input::new()
            .with_prompt("Host")
            .default(session.host)
            .interact_text()?;

        let user: String = Input::new()
            .with_prompt("Username")
            .default(session.user)
            .interact_text()?;

        let port: u16 = Input::new()
            .with_prompt("Port")
            .default(session.port)
            .interact_text()?;

        let auth_types = vec!["key", "password"];
        let auth_type_idx = Select::new()
            .with_prompt("Authentication type")
            .items(&auth_types)
            .default(match session.auth_type {
                AuthType::Key => 0,
                AuthType::Password => 1,
            })
            .interact()?;

        let (auth_type, private_key_path, password) = match auth_types[auth_type_idx] {
            "key" => {
                let key_path: String = Input::new()
                    .with_prompt("Private key path")
                    .default(
                        session
                            .private_key_path
                            .unwrap_or_else(|| PathBuf::from("~/.ssh/id_rsa"))
                            .to_string_lossy()
                            .to_string(),
                    )
                    .interact_text()?;
                (AuthType::Key, Some(PathBuf::from(key_path)), None)
            }
            "password" => {
                let password = read_password().context("Failed to read password")?;
                (AuthType::Password, None, Some(password))
            }
            _ => unreachable!(),
        };

        let tags_input: String = Input::new()
            .with_prompt("Tags (comma or semicolon separated)")
            .default(session.tags.join(", "))
            .allow_empty(true)
            .interact_text()?;

        let tags = if tags_input.is_empty() {
            None
        } else {
            Some(parse_tags(Some(&tags_input)))
        };

        Session::new(
            session.name,
            host,
            user,
            port,
            auth_type,
            private_key_path,
            password,
            tags,
        )
    };

    new_session.validate()?;
    config.update_session(new_session)?;
    println!("Session '{}' modified successfully.", name);
    Ok(())
}

async fn handle_login(name: Option<String>, tags: Option<String>) -> Result<()> {
    let config = Config::load()?;

    let session = match name {
        Some(name) => config
            .get_session(&name)
            .context("Session not found")?
            .clone(),
        None => {
            if config.sessions.is_empty() {
                anyhow::bail!("No SSH sessions found");
            }

            // Filter sessions by tags if specified
            let filtered_sessions: Vec<&Session> = if let Some(tags_str) = tags {
                let filter_tags: HashSet<String> =
                    parse_tags(Some(&tags_str)).into_iter().collect();
                config
                    .sessions
                    .iter()
                    .filter(|session| {
                        let session_tags: HashSet<String> = session.tags.iter().cloned().collect();
                        !filter_tags.is_disjoint(&session_tags)
                    })
                    .collect()
            } else {
                config.sessions.iter().collect()
            };

            if filtered_sessions.is_empty() {
                anyhow::bail!("No SSH sessions found matching the specified tags");
            }

            let session_names: Vec<String> = filtered_sessions
                .iter()
                .map(|s| {
                    let tags_str = if s.tags.is_empty() {
                        "".to_string()
                    } else {
                        format!(" [{}]", s.tags.join(", "))
                    };
                    format!("{} ({}@{}:{}){}", s.name, s.user, s.host, s.port, tags_str)
                })
                .collect();

            let selection = Select::new()
                .with_prompt("Select a session")
                .items(&session_names)
                .default(0)
                .interact()?;

            filtered_sessions[selection].clone()
        }
    };

    // Use the SSH utility module to connect
    ssh::connect_ssh(&session)
}

fn handle_tag(name: String, action: String, tags: Option<String>) -> Result<()> {
    let mut config = Config::load()?;
    let session = config
        .get_session(&name)
        .context("Session not found")?
        .clone();

    let mut session_tags: HashSet<String> = session.tags.iter().cloned().collect();

    match action.to_lowercase().as_str() {
        "add" => {
            if let Some(tags_str) = tags {
                let new_tags = parse_tags(Some(&tags_str));
                session_tags.extend(new_tags);
                println!("Tags added to session '{}'.", name);
            } else {
                anyhow::bail!("Tags must be specified for 'add' action");
            }
        }
        "remove" => {
            if let Some(tags_str) = tags {
                let tags_to_remove = parse_tags(Some(&tags_str));
                session_tags.retain(|tag| !tags_to_remove.contains(tag));
                println!("Tags removed from session '{}'.", name);
            } else {
                anyhow::bail!("Tags must be specified for 'remove' action");
            }
        }
        "list" => {
            if session_tags.is_empty() {
                println!("Session '{}' has no tags.", name);
            } else {
                let tags_vec: Vec<String> = session_tags.iter().cloned().collect();
                println!("Tags for session '{}': {}", name, tags_vec.join(", "));
            }
            return Ok(());
        }
        _ => anyhow::bail!("Invalid action. Use 'add', 'remove', or 'list'"),
    }

    let mut updated_session = session;
    updated_session.tags = session_tags.into_iter().collect();
    config.update_session(updated_session)?;

    Ok(())
}
