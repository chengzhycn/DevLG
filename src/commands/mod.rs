use anyhow::Result;
use clap::{Parser, Subcommand};
use std::{collections::HashSet, path::PathBuf};

mod cp;
mod delete;
mod list;
mod login;
mod modify;
mod tag;
mod template;
mod version;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Show the current version
    Version,

    /// List all SSH sessions
    List {
        /// Show detailed information about each session
        #[arg(short, long)]
        detailed: bool,

        /// Filter sessions by tags (comma or semicolon separated)
        #[arg(short, long)]
        tags: Option<String>,
    },

    /// Add a new SSH session
    Add {
        /// Session name
        #[arg(short, long)]
        name: Option<String>,

        /// Host address
        #[arg(short = 'H', long)]
        host: Option<String>,

        /// Username
        #[arg(short, long, default_value = "root")]
        user: Option<String>,

        /// SSH port
        #[arg(short, long, default_value = "22")]
        port: Option<u16>,

        /// Authentication type (key or password)
        #[arg(short, long, default_value = "key")]
        auth_type: Option<String>,

        /// Path to private key file
        #[arg(short = 'k', long)]
        key_path: Option<PathBuf>,

        /// Password for authentication
        #[arg(short = 'P', long)]
        password: Option<String>,

        /// Tags for the session (comma or semicolon separated)
        #[arg(short, long)]
        tags: Option<String>,

        /// Template name to use as base
        #[arg(short = 'T', long)]
        template: Option<String>,
    },

    /// Delete an SSH session
    Delete {
        /// Session names to delete
        names: Vec<String>,

        /// Delete sessions by tag, if provided, names will be ignored
        #[arg(short, long)]
        tag: Option<String>,
    },

    /// Modify an existing SSH session
    Modify {
        /// Session name to modify
        name: String,

        /// New host address
        #[arg(short = 'H', long)]
        host: Option<String>,

        /// New username
        #[arg(short, long)]
        user: Option<String>,

        /// New SSH port
        #[arg(short, long)]
        port: Option<u16>,

        /// New authentication type (key or password)
        #[arg(short, long)]
        auth_type: Option<String>,

        /// New path to private key file
        #[arg(short = 'k', long)]
        key_path: Option<PathBuf>,

        /// New password for authentication
        #[arg(short = 'P', long)]
        password: Option<String>,

        /// New tags for the session (comma or semicolon separated)
        #[arg(short, long)]
        tags: Option<String>,
    },

    /// Login to an SSH session
    Login {
        /// Session name to login to
        name: Option<String>,

        /// Filter sessions by tags (comma or semicolon separated)
        #[arg(short, long)]
        tags: Option<String>,
    },

    /// Manage tags for SSH sessions
    Tag {
        /// Session name
        name: String,

        /// Action to perform (add, remove, list)
        #[arg(short, long)]
        action: String,

        /// Tags to add or remove (comma or semicolon separated)
        #[arg(short, long)]
        tags: Option<String>,
    },

    /// Manage SSH session templates
    Template {
        #[command(subcommand)]
        action: TemplateAction,
    },

    /// Copy files between SSH sessions and local.
    Cp {
        /// Source/destination file or directory. Can use [local_path] or [session_name]:[remote_path]
        /// The last path is the destination, the rest are sources.
        paths: Vec<PathBuf>,

        /// copy files from the remote source to the local destination
        #[arg(short, long, conflicts_with = "dst")]
        src: Option<String>,

        /// copy files from the local source to the remote destination
        #[arg(short, long, conflicts_with = "src")]
        dst: Option<String>,

        /// Recursively copy directories
        #[arg(short, long)]
        recursive: bool,
    },
}

#[derive(Subcommand)]
pub enum TemplateAction {
    /// List all templates
    List,

    /// Delete a template
    Delete {
        /// Template name to delete
        name: String,
    },

    /// Add a template from an existing session
    Add {
        /// Template name
        name: String,

        /// Session name to use as template
        #[arg(short, long)]
        session: String,
    },
}

#[derive(Default)]
struct SessionParams {
    name: Option<String>,
    host: Option<String>,
    user: Option<String>,
    port: Option<u16>,
    auth_type: Option<String>,
    key_path: Option<PathBuf>,
    password: Option<String>,
    tags: Option<String>,
}

impl SessionParams {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: Option<String>,
        host: Option<String>,
        user: Option<String>,
        port: Option<u16>,
        auth_type: Option<String>,
        key_path: Option<PathBuf>,
        password: Option<String>,
        tags: Option<String>,
    ) -> Self {
        Self {
            name,
            host,
            user,
            port,
            auth_type,
            key_path,
            password,
            tags,
        }
    }
}

pub async fn handle_command(command: Commands) -> Result<()> {
    match command {
        Commands::Version => version::handle_version(),
        Commands::List { detailed, tags } => list::handle_list(detailed, tags),
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
                let params =
                    SessionParams::new(name, host, user, port, auth_type, key_path, password, tags);
                modify::handle_add(params).await
            } else {
                modify::handle_add_with_template(template.unwrap()).await
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
            let params = SessionParams::new(
                Some(name),
                host,
                user,
                port,
                auth_type,
                key_path,
                password,
                tags,
            );
            modify::handle_modify(params).await
        }
        Commands::Delete { names, tag } => {
            match tag {
                Some(tag) => {
                    // TODO: validate tag format
                    delete::handle_delete_with_tags(tag).await
                }
                None => delete::handle_delete(names).await,
            }
        }
        Commands::Login { name, tags } => login::handle_login(name, tags).await,
        Commands::Tag { name, action, tags } => tag::handle_tag(name, action, tags),
        Commands::Template { action } => match action {
            TemplateAction::List => template::handle_template_list().await,
            TemplateAction::Add { session, name } => {
                template::handle_template_add(name, session).await
            }
            TemplateAction::Delete { name } => template::handle_template_delete(name).await,
        },
        Commands::Cp {
            paths,
            src,
            dst,
            recursive,
        } => cp::handle_cp(paths, src, dst, recursive).await,
    }
}

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
