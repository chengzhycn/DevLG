# DevLG - SSH Session Manager

DevLG is a command-line SSH session manager written in Rust. It helps you manage multiple SSH connections efficiently by storing configurations in a TOML file and providing an intuitive interface for managing and connecting to remote servers.

## Features

- Manage SSH sessions (add, modify, delete, list)
- Interactive and command-line modes for adding new sessions
- Support for both password and private key authentication
- Configuration stored in TOML format
- Interactive session selection for quick login
- Secure credential storage

## Prerequisites

- Rust 1.70 or later
- OpenSSH client
- A Unix-like operating system (Linux, macOS, etc.)

## Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/devlg.git
cd devlg

# Build and install
cargo install --path .
```

## Usage

### Basic Commands

```bash
# List all SSH sessions
devlg list

# Add a new SSH session interactively
devlg add

# Add a new SSH session from command line
devlg add --name myserver --host example.com --user username

# Login to a specific session
devlg login myserver

# Login with interactive session selection
devlg login

# Delete a session
devlg delete myserver

# Modify a session
devlg modify myserver
```

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

[[sessions]]
name = "password-server"
host = "example.org"
user = "admin"
port = 22
auth_type = "password"
password = "your-password"
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

- [ ] Session grouping/categorization
- [ ] Session search functionality
- [ ] Session import/export
- [ ] Session templates
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
