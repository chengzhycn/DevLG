use anyhow::Result;
use std::collections::HashSet;

use crate::config::manager::ConfigManager;
use crate::models::session::Session;

pub fn handle_delete(names: Vec<String>) -> Result<()> {
    let mut manager = ConfigManager::new(None);
    manager.load()?;

    for name in names {
        manager.config.remove_session(&name)?;
        println!("Session '{}' deleted successfully.", name);
    }
    manager.save()?;

    Ok(())
}

pub fn handle_delete_with_tags(tags: String) -> Result<()> {
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
