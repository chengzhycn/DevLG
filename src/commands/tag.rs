use anyhow::{Context, Result};
use std::collections::HashSet;

use crate::commands::parse_tags;
use crate::config::manager::ConfigManager;

pub fn handle_tag(name: String, action: String, tags: Option<String>) -> Result<()> {
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
