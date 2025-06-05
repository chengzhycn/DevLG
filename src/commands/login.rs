use anyhow::Result;
use dialoguer::Select;
use std::collections::HashSet;

use crate::commands::parse_tags;
use crate::config::manager::ConfigManager;
use crate::models::session::Session;
use crate::utils::ssh;

pub async fn handle_login(name: Option<String>, tags: Option<String>) -> Result<()> {
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
