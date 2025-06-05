use anyhow::Result;
use std::collections::HashSet;

use crate::commands::parse_tags;
use crate::config::manager::ConfigManager;
use crate::models::session::Session;

pub fn handle_list(detailed: bool, tags_filter: Option<String>) -> Result<()> {
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
