use clap::{Parser, Subcommand};
use std::path::PathBuf;

pub mod handlers;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
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

    /// Create a template from an existing session
    Create {
        /// Template name
        name: String,

        /// Session name to use as template
        #[arg(short, long)]
        session: String,
    },
}
