use anyhow::{Context, Ok, Result};
use dialoguer::{Input, Select};
use rpassword::read_password;
use std::path::PathBuf;

use crate::commands::{SessionParams, parse_tags};
use crate::config::manager::ConfigManager;
use crate::models::session::{AuthType, Session};

pub async fn handle_add(params: SessionParams) -> Result<()> {
    let mut manager = ConfigManager::new(None);
    manager.load()?;

    let session = if params.name.is_some() && params.host.is_some() && params.user.is_some() {
        // Command line mode
        // auth_type has a default value of "key", so it can safely be unwrapped
        let auth_type = params.auth_type.unwrap().parse()?;

        Session::new(
            params.name.unwrap(),
            params.host.unwrap(),
            params.user.unwrap(),
            params.port.unwrap(),
            auth_type,
            params.key_path,
            params.password,
            Some(parse_tags(params.tags.as_ref())),
        )
    } else {
        // Interactive mode
        new_session_with_default(&Session::empty_template(), true).await?
    };

    session.validate()?;
    manager.config.add_session(session)?;
    manager.save()?;
    println!("Session added successfully.");
    Ok(())
}

pub async fn handle_add_with_template(name: String) -> Result<()> {
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
    let new_session = new_session_with_default(session, true).await?;

    new_session.validate()?;
    manager.config.add_session(new_session)?;
    manager.save()?;
    println!("Session added successfully.");
    Ok(())
}

async fn new_session_with_default(sess: &Session, create: bool) -> Result<Session> {
    let name = if create {
        Input::new()
            .with_prompt("Session name")
            .default(sess.name.clone())
            .interact_text()?
    } else {
        sess.name.clone()
    };

    let host: String = Input::new()
        .with_prompt("Host")
        .default(sess.host.clone())
        .interact_text()?;

    let user: String = Input::new()
        .with_prompt("Username")
        .default(sess.user.clone())
        .interact_text()?;

    let port: u16 = Input::new()
        .with_prompt("Port")
        .default(sess.port)
        .interact_text()?;

    let auth_types = vec![AuthType::Key, AuthType::Password];
    let auth_type_idx = Select::new()
        .with_prompt("Authentication type")
        .items(&auth_types)
        .default(match sess.auth_type {
            AuthType::Key => 0,
            AuthType::Password => 1,
        })
        .interact()?;
    let (auth_type, private_key_path, password) = match auth_types[auth_type_idx] {
        AuthType::Key => {
            let key_path: String = Input::new()
                .with_prompt("Private key path")
                .default(
                    sess.private_key_path
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
                sess.password.clone()
            } else {
                Some(new_pass)
            };

            (AuthType::Password, None, password)
        }
    };

    let tags_input: String = Input::new()
        .with_prompt("Tags (comma or semicolon separated)")
        .default(
            sess.tags
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

    Ok(new_session)
}

pub async fn handle_modify(params: SessionParams) -> Result<()> {
    let mut manager = ConfigManager::new(None);
    manager.load()?;

    let session = manager
        .config
        .get_session(&params.name.unwrap())
        .context("Session not found")?
        .clone();

    let new_session = if params.host.is_some()
        || params.user.is_some()
        || params.port.is_some()
        || params.auth_type.is_some()
        || params.key_path.is_some()
        || params.password.is_some()
        || params.tags.is_some()
    {
        // Command line mode
        // auth_type has a default value of "key", so it can safely be unwrapped
        let auth_type = params.auth_type.unwrap().parse()?;

        Session::new(
            session.name,
            params.host.unwrap_or(session.host),
            params.user.unwrap_or(session.user),
            params.port.unwrap_or(session.port),
            auth_type,
            params.key_path.or(session.private_key_path),
            params.password.or(session.password),
            Some(
                params
                    .tags
                    .map_or_else(|| session.tags.clone(), |s| parse_tags(Some(&s))),
            ),
        )
    } else {
        // Interactive mode
        new_session_with_default(&session, false).await?
    };

    new_session.validate()?;
    manager.config.update_session(new_session)?;
    manager.save()?;
    println!("Session modified successfully.");
    Ok(())
}
