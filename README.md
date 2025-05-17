# DevLG - SSH Session Manager

DevLG is a command-line SSH session manager written in Rust. It helps you manage multiple SSH connections efficiently by storing configurations in a TOML file and providing an intuitive interface for managing and connecting to remote servers.

## Features

- Manage SSH sessions (add, modify, delete, list)
- Interactive and command-line modes for adding new sessions
- Support for both password and private key authentication
- Configuration stored in TOML format
- Interactive session selection for quick login
- Secure credential storage
- Tag-based session organization and filtering
- Session templates for quick session creation

## Prerequisites

- Rust 1.70 or later
- OpenSSH client
- A Unix-like operating system (Linux, macOS, etc.)

## Installation

You can install DevLG using the following one-line command:

```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/chengzhycn/devlg/main/install.sh)"
```

Or manually:

```bash
# Clone the repository
git clone https://github.com/chengzhycn/devlg.git
cd devlg

# Build and install
cargo install --path .
```

## Usage

### Basic Commands

```bash
# List all SSH sessions
devlg list

# List sessions with detailed information
devlg list --detailed

# List sessions filtered by tags
devlg list --tags "production,web"

# Add a new SSH session interactively
devlg add

# Add a new SSH session from command line
devlg add --name myserver --host example.com --user username --tags "production,web"

# Add a new SSH session using a template
devlg add --template mytemplate

# Login to a specific session
devlg login myserver

# Login with interactive session selection
devlg login

# Delete a session
devlg delete myserver

# Modify a session
devlg modify myserver

# Manage session tags
devlg tag myserver --action add --tags "production,web"
devlg tag myserver --action remove --tags "web"
devlg tag myserver --action list
```

### Template Management

DevLG supports session templates to quickly create new sessions with predefined settings:

1. **List Templates**: View all available templates.

   ```bash
   devlg template list
   ```

2. **Create a Template**: Create a new template from an existing session.

   ```bash
   devlg template create --session myserver mytemplate
   ```

3. **Delete a Template**: Remove an existing template.

   ```bash
   devlg template delete mytemplate
   ```

4. **Using Templates**: When adding a new session, you can use a template as a base.

   ```bash
   # Use a template with command line
   devlg add --template mytemplate
   ```

   When using a template, the new session will inherit all properties from the template's source session, but you can override any of them in the interactive mode. The session name will be prompted with the template's source session name as the default value.

### Configuration

The configuration file is stored at `~/.config/devlg.toml`. Here's an example configuration:

```toml
[[sessions]]
name = "myserver"
host = "example.com"
user = "username"
port = 22
auth_type = "key"
private_key_path = "~/.ssh/id_rsa"
tags = ["production", "web"]

[[sessions]]
name = "password-server"
host = "example.org"
user = "admin"
port = 22
auth_type = "password"
password = "your-password"
tags = ["staging", "database"]

[[templates]]
name = "production-template"
host = "prod.example.com"
user = "deploy"
port = 22
auth_type = "key"
private_key_path = "~/.ssh/deploy_key"
tags = ["production"]
```

## Tag Management

DevLG supports tagging SSH sessions for better organization and filtering:

1. **Adding Tags**: When creating or modifying a session, you can specify tags using the `--tags` option with comma or semicolon-separated values.

   ```bash
   devlg add --name myserver --host example.com --tags "production,web"
   ```

2. **Managing Tags**: Use the `tag` command to add, remove, or list tags for a session.

   ```bash
   # Add tags
   devlg tag myserver --action add --tags "production,web"

   # Remove tags
   devlg tag myserver --action remove --tags "web"

   # List tags
   devlg tag myserver --action list
   ```

3. **Filtering by Tags**: When listing sessions, you can filter by tags using the `--tags` option.

   ```bash
   # Show only production servers
   devlg list --tags "production"

   # Show servers with either production or web tags
   devlg list --tags "production,web"
   ```

## Development Roadmap

### Phase 1: Core Functionality

- [x] Project setup and basic structure
- [x] Configuration file management
- [x] Session CRUD operations
- [x] Basic CLI interface
- [x] Interactive session management
- [x] Command-line parsing
- [x] Session selection interface

### Phase 2: Authentication

- [x] Password authentication support
- [x] Private key authentication support
- [ ] Secure credential storage (encryption)
- [ ] SSH connection implementation
  - [ ] Basic connection using ssh2 crate
  - [ ] Password authentication
  - [ ] Private key authentication
  - [ ] Connection error handling
  - [ ] Interactive shell support

### Phase 3: User Experience

- [x] Session grouping/categorization with tags
- [x] Session filtering by tags
- [ ] Session search functionality
- [ ] Session import/export
- [x] Session templates
- [ ] Command completion
- [ ] Progress indicators for long operations

### Phase 4: Testing and Documentation

- [x] Basic unit tests
- [ ] Integration tests
- [ ] End-to-end tests
- [ ] Documentation for each module
- [ ] Usage examples
- [ ] Troubleshooting guide

### Phase 5: Security Enhancements

- [ ] Password encryption
- [ ] Secure credential storage
- [ ] Audit logging
- [ ] Session timeout
- [ ] Connection verification

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
