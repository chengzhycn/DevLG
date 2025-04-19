use crate::commands::Commands;
use crate::config::manager::Config;
use crate::models::session::{AuthType, Session};
use crate::utils::ssh;
use anyhow::{Context, Result};
use dialoguer::{Input, Select};
use rpassword::read_password;
use std::path::PathBuf;

pub async fn handle_command(command: Commands) -> Result<()> {
    match command {
        Commands::List { detailed } => handle_list(detailed),
        Commands::Add {
            name,
            host,
            user,
            port,
            auth_type,
            key_path,
            password,
        } => handle_add(name, host, user, port, auth_type, key_path, password).await,
        Commands::Delete { name } => handle_delete(name),
        Commands::Modify {
            name,
            host,
            user,
            port,
            auth_type,
            key_path,
            password,
        } => handle_modify(name, host, user, port, auth_type, key_path, password).await,
        Commands::Login { name } => handle_login(name).await,
    }
}

fn handle_list(detailed: bool) -> Result<()> {
    let config = Config::load()?;
    if config.sessions.is_empty() {
        println!("No SSH sessions found.");
        return Ok(());
    }

    println!("Available SSH sessions:");
    if detailed {
        println!(
            "{:<20} {:<15} {:<10} {:<6} {:<10} {:<20}",
            "Name", "Host", "User", "Port", "Auth Type", "Key Path"
        );
        println!("{:-<85}", "");

        for session in config.sessions.iter() {
            let auth_type = match session.auth_type {
                AuthType::Key => "Key",
                AuthType::Password => "Password",
            };

            let key_path = session
                .private_key_path
                .as_ref()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| "N/A".to_string());

            println!(
                "{:<20} {:<15} {:<10} {:<6} {:<10} {:<20}",
                session.name, session.host, session.user, session.port, auth_type, key_path
            );
        }
    } else {
        for (i, session) in config.sessions.iter().enumerate() {
            println!(
                "{}. {} ({}@{}:{})",
                i + 1,
                session.name,
                session.user,
                session.host,
                session.port
            );
        }
    }
    Ok(())
}

async fn handle_add(
    name: Option<String>,
    host: Option<String>,
    user: Option<String>,
    port: Option<u16>,
    auth_type: Option<String>,
    key_path: Option<PathBuf>,
    password: Option<String>,
) -> Result<()> {
    let mut config = Config::load()?;

    let session = if name.is_some() && host.is_some() && user.is_some() {
        // Command line mode
        let auth_type = match auth_type.unwrap_or_else(|| "key".to_string()).as_str() {
            "key" => AuthType::Key,
            "password" => AuthType::Password,
            _ => anyhow::bail!("Invalid authentication type. Use 'key' or 'password'"),
        };

        Session::new(
            name.unwrap(),
            host.unwrap(),
            user.unwrap(),
            port.unwrap_or(22),
            auth_type,
            key_path,
            password,
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

        Session::new(
            name,
            host,
            user,
            port,
            auth_type,
            private_key_path,
            password,
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

async fn handle_modify(
    name: String,
    host: Option<String>,
    user: Option<String>,
    port: Option<u16>,
    auth_type: Option<String>,
    key_path: Option<PathBuf>,
    password: Option<String>,
) -> Result<()> {
    let mut config = Config::load()?;
    let session = config
        .get_session(&name)
        .context("Session not found")?
        .clone();

    let new_session = if host.is_some()
        || user.is_some()
        || port.is_some()
        || auth_type.is_some()
        || key_path.is_some()
        || password.is_some()
    {
        // Command line mode
        let auth_type = match auth_type
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
            host.unwrap_or(session.host),
            user.unwrap_or(session.user),
            port.unwrap_or(session.port),
            auth_type,
            key_path.or(session.private_key_path),
            password.or(session.password),
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

        Session::new(
            session.name,
            host,
            user,
            port,
            auth_type,
            private_key_path,
            password,
        )
    };

    new_session.validate()?;
    config.update_session(new_session)?;
    println!("Session '{}' modified successfully.", name);
    Ok(())
}

async fn handle_login(name: Option<String>) -> Result<()> {
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

            let session_names: Vec<String> = config
                .sessions
                .iter()
                .map(|s| format!("{} ({}@{}:{})", s.name, s.user, s.host, s.port))
                .collect();

            let selection = Select::new()
                .with_prompt("Select a session")
                .items(&session_names)
                .default(0)
                .interact()?;

            config.sessions[selection].clone()
        }
    };

    // Use the SSH utility module to connect
    ssh::connect_ssh(&session)
}
