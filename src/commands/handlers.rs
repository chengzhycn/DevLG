use crate::commands::Commands;
use crate::config::manager::ConfigManager;
use crate::models::session::{AuthType, Session, Template};
use crate::utils::{scp, ssh};
use anyhow::{Context, Ok, Result};
use dialoguer::{Input, Select};
use rpassword::read_password;
use std::collections::HashSet;
use std::path::PathBuf;

use super::TemplateAction;

// Helper function to parse tags from a string
fn parse_tags(tags_str: Option<&String>) -> HashSet<String> {
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
        Commands::Version => handle_version(),
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
            template,
        } => {
            if template.is_none() {
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
            } else {
                handle_add_with_template(template.unwrap()).await
            }
        }
        Commands::Delete { names, tag } => {
            match tag {
                Some(tag) => {
                    // TODO: validate tag format
                    handle_delete_with_tags(tag).await
                }
                None => handle_delete(names).await,
            }
        }
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
        Commands::Template { action } => match action {
            TemplateAction::List => handle_template_list().await,
            TemplateAction::Create { session, name } => handle_template_add(name, session).await,
            TemplateAction::Delete { name } => handle_template_delete(name).await,
        },
        Commands::Cp {
            src,
            dst,
            recursive,
        } => handle_cp(src, dst, recursive).await,
    }
}

fn handle_version() -> Result<()> {
    println!("devlg version {}", env!("CARGO_PKG_VERSION"));
    Ok(())
}

fn handle_list(detailed: bool, tags_filter: Option<String>) -> Result<()> {
    let mut manager = ConfigManager::new(None);
    manager.load()?;

    let config = manager.config;
    if config.sessions.is_empty() {
        println!("No SSH sessions found.");
        return Ok(());
    }

    // Filter sessions by tags if specified
    let filtered_sessions: Vec<&Session> = if let Some(tags_str) = tags_filter {
        let filter_tags: HashSet<String> = parse_tags(Some(&tags_str));
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
            let auth_type = session.auth_type.to_string();

            let key_path = session
                .private_key_path
                .as_ref()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| "N/A".to_string());

            let tags_str = if session.tags.is_empty() {
                "N/A".to_string()
            } else {
                session
                    .tags
                    .iter()
                    .cloned()
                    .collect::<Vec<String>>()
                    .join(", ")
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
                format!(
                    " [{}]",
                    session
                        .tags
                        .iter()
                        .cloned()
                        .collect::<Vec<String>>()
                        .join(", ")
                )
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
    let mut manager = ConfigManager::new(None);
    manager.load()?;

    let session = if addition.name.is_some() && addition.host.is_some() && addition.user.is_some() {
        // Command line mode
        // auth_type has a default value of "key", so it can safely be unwrapped
        let auth_type = addition.auth_type.unwrap().parse()?;

        Session::new(
            addition.name.unwrap(),
            addition.host.unwrap(),
            addition.user.unwrap(),
            addition.port.unwrap(),
            auth_type,
            addition.key_path,
            addition.password,
            Some(parse_tags(addition.tags.as_ref())),
        )
    } else {
        // Interactive mode
        let name: String = Input::new().with_prompt("Session name").interact_text()?;

        let host: String = Input::new().with_prompt("Host").interact_text()?;

        let user: String = Input::new()
            .with_prompt("Username")
            .default("root".to_string())
            .interact_text()?;

        let port: u16 = Input::new()
            .with_prompt("Port")
            .default(22)
            .interact_text()?;

        let auth_types = vec![AuthType::Key, AuthType::Password];
        let auth_type_idx = Select::new()
            .with_prompt("Authentication type")
            .items(&auth_types)
            .default(0)
            .interact()?;

        let (auth_type, private_key_path, password) = match auth_types[auth_type_idx] {
            AuthType::Key => {
                let key_path: String = Input::new()
                    .with_prompt("Private key path")
                    .default("~/.ssh/id_rsa".to_string())
                    .interact_text()?;
                (AuthType::Key, Some(PathBuf::from(key_path)), None)
            }
            AuthType::Password => {
                let password = read_password().context("Failed to read password")?;
                (AuthType::Password, None, Some(password))
            }
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
    manager.config.add_session(session)?;
    manager.save()?;
    println!("Session added successfully.");
    Ok(())
}

async fn handle_add_with_template(name: String) -> Result<()> {
    let mut manager = ConfigManager::new(None);
    manager.load()?;

    let template = manager
        .config
        .get_template(&name)
        .context("Template not found")?;

    let session = manager
        .config
        .get_session(&template.session)
        .context("Session not found")?;

    // enter interactive mode
    let name: String = Input::new()
        .with_prompt("Session name")
        .default(session.name.clone())
        .interact_text()?;

    let host: String = Input::new()
        .with_prompt("Host")
        .default(session.host.clone())
        .interact_text()?;

    let user: String = Input::new()
        .with_prompt("Username")
        .default(session.user.clone())
        .interact_text()?;

    let port: u16 = Input::new()
        .with_prompt("Port")
        .default(session.port)
        .interact_text()?;

    let auth_types = vec![AuthType::Key, AuthType::Password];
    let auth_type_idx = Select::new()
        .with_prompt("Authentication type")
        .items(&auth_types)
        .default(match session.auth_type {
            AuthType::Key => 0,
            AuthType::Password => 1,
        })
        .interact()?;
    let (auth_type, private_key_path, password) = match auth_types[auth_type_idx] {
        AuthType::Key => {
            let key_path: String = Input::new()
                .with_prompt("Private key path")
                .default(
                    session
                        .private_key_path
                        .clone()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                )
                .interact_text()?;
            (AuthType::Key, Some(PathBuf::from(key_path)), None)
        }
        AuthType::Password => {
            let new_pass = read_password().context("Failed to read password")?;
            let password = if new_pass.is_empty() {
                session.password.clone()
            } else {
                Some(new_pass)
            };

            (AuthType::Password, None, password)
        }
    };

    let tags_input: String = Input::new()
        .with_prompt("Tags (comma or semicolon separated)")
        .default(
            session
                .tags
                .iter()
                .cloned()
                .collect::<Vec<String>>()
                .join(", "),
        )
        .allow_empty(true)
        .interact_text()?;

    let tags = if tags_input.is_empty() {
        None
    } else {
        Some(parse_tags(Some(&tags_input)))
    };

    let new_session = Session::new(
        name,
        host,
        user,
        port,
        auth_type,
        private_key_path,
        password,
        tags,
    );

    new_session.validate()?;
    manager.config.add_session(new_session)?;
    manager.save()?;
    println!("Session added successfully.");
    Ok(())
}

async fn handle_delete(names: Vec<String>) -> Result<()> {
    let mut manager = ConfigManager::new(None);
    manager.load()?;

    for name in names {
        manager.config.remove_session(&name)?;
        println!("Session '{}' deleted successfully.", name);
    }
    manager.save()?;

    Ok(())
}

async fn handle_delete_with_tags(tags: String) -> Result<()> {
    let mut manager = ConfigManager::new(None);
    manager.load()?;

    let sessions: Vec<Session> = manager
        .config
        .sessions
        .iter()
        .filter(|session| {
            let session_tags: HashSet<String> = session.tags.iter().cloned().collect();
            if session_tags.contains(&tags) {
                println!("Session '{}' deleted successfully.", session.name);
                false
            } else {
                true
            }
        })
        .cloned()
        .collect();

    manager.config.sessions = sessions;
    manager.save()?;

    Ok(())
}

async fn handle_modify(name: String, modification: SessionModification) -> Result<()> {
    let mut manager = ConfigManager::new(None);
    manager.load()?;

    let session = manager
        .config
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
        // auth_type has a default value of "key", so it can safely be unwrapped
        let auth_type = modification.auth_type.unwrap().parse()?;

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

        let auth_types = vec![AuthType::Key, AuthType::Password];
        let auth_type_idx = Select::new()
            .with_prompt("Authentication type")
            .items(&auth_types)
            .default(match session.auth_type {
                AuthType::Key => 0,
                AuthType::Password => 1,
            })
            .interact()?;

        let (auth_type, private_key_path, password) = match auth_types[auth_type_idx] {
            AuthType::Key => {
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
            AuthType::Password => {
                let password = read_password().context("Failed to read password")?;
                (AuthType::Password, None, Some(password))
            }
        };

        let tags_input: String = Input::new()
            .with_prompt("Tags (comma or semicolon separated)")
            .default(
                session
                    .tags
                    .iter()
                    .cloned()
                    .collect::<Vec<String>>()
                    .join(", "),
            )
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
    manager.config.update_session(new_session)?;
    manager.save()?;
    println!("Session '{}' modified successfully.", name);
    Ok(())
}

async fn handle_login(name: Option<String>, tags: Option<String>) -> Result<()> {
    let mut manager = ConfigManager::new(None);
    manager.load()?;
    let config = manager.config;

    let session = match name {
        Some(name) => {
            let sessions = config.search_sessions(&name, &tags.unwrap_or_default());
            if sessions.is_empty() {
                anyhow::bail!("No SSH sessions found matching the specified name")
            }
            if sessions.len() == 1 {
                println!(
                    "Found session {} matching the specified name",
                    sessions[0].name
                );
                sessions[0].clone()
            } else {
                let session_names: Vec<String> = sessions
                    .iter()
                    .map(|s| {
                        let tags_str = if s.tags.is_empty() {
                            "".to_string()
                        } else {
                            format!(
                                " [{}]",
                                s.tags.iter().cloned().collect::<Vec<String>>().join(", ")
                            )
                        };
                        format!("{} ({}@{}:{}){}", s.name, s.user, s.host, s.port, tags_str)
                    })
                    .collect();

                let selection = Select::new()
                    .with_prompt("Select a session")
                    .items(&session_names)
                    .default(0)
                    .interact()?;

                sessions[selection].clone()
            }
        }
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
                        format!(
                            " [{}]",
                            s.tags.iter().cloned().collect::<Vec<String>>().join(", ")
                        )
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
    let mut manager = ConfigManager::new(None);
    manager.load()?;

    let session = manager
        .config
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
    manager.config.update_session(updated_session)?;
    manager.save()?;

    Ok(())
}

async fn handle_template_list() -> Result<()> {
    let mut manager = ConfigManager::new(None);
    manager.load()?;

    println!("Available templates:");
    for template in manager.config.templates.iter() {
        println!("{}", template.name);
    }

    Ok(())
}

async fn handle_template_add(name: String, session: String) -> Result<()> {
    let mut manager = ConfigManager::new(None);
    manager.load()?;

    manager.config.add_template(Template { name, session })?;
    manager.save()?;
    println!("Template added successfully.");
    Ok(())
}

async fn handle_template_delete(name: String) -> Result<()> {
    let mut manager = ConfigManager::new(None);
    manager.load()?;
    manager.config.remove_template(&name)?;
    manager.save()?;
    println!("Template deleted successfully.");
    Ok(())
}

async fn handle_cp(src: String, dst: String, recursive: bool) -> Result<()> {
    let mut manager = ConfigManager::new(None);
    manager.load()?;

    let (src_session_name, src_path) = parse_path(src);
    let (dst_session_name, dst_path) = parse_path(dst);

    let src_session = src_session_name
        .as_ref()
        .and_then(|name| manager.config.get_session(name));

    let dst_session = dst_session_name
        .as_ref()
        .and_then(|name| manager.config.get_session(name));

    scp::copy_file(src_session, dst_session, &src_path, &dst_path, recursive)
}

fn parse_path(path: String) -> (Option<String>, PathBuf) {
    match path.split_once(":") {
        Some((session_name, file_path)) => {
            (Some(session_name.to_string()), PathBuf::from(file_path))
        }
        None => (None, PathBuf::from(path)),
    }
}
