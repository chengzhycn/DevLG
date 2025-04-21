# DevLG - SSH Session Manager

DevLG is a command-line SSH session manager written in Rust. It helps you manage multiple SSH connections efficiently by storing configurations in a TOML file and providing an intuitive interface for managing and connecting to remote servers.

## Features

- Manage SSH sessions (add, modify, delete, list)
- Interactive and command-line modes for adding new sessions
- Support for both password and private key authentication
- Configuration stored in TOML format
- Interactive session selection for quick login
- Multiple SSH connection implementations (system SSH client and ssh2 crate)
- Secure credential storage

## Prerequisites

- Rust 1.70 or later
- OpenSSH client (for system SSH client option)
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

# Login to a specific session (using ssh2 crate by default)
devlg login myserver

# Login with interactive session selection
devlg login

# Login using system SSH client
devlg login myserver --use-system-ssh

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

## SSH Connection Implementations

DevLG provides two different SSH connection implementations:

1. **Ssh2Connector (Default)**: Uses the ssh2 crate for SSH connections

   - Pure Rust implementation
   - No dependency on system SSH client
   - Better error handling
   - More control over the connection

2. **SystemSshConnector**: Uses the system's SSH client
   - Relies on OpenSSH client
   - Uses sshpass for password authentication
   - More compatible with existing SSH configurations

You can choose which implementation to use with the `--use-system-ssh` flag when logging in.

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
- [x] SSH connection implementation
  - [x] Basic connection using ssh2 crate
  - [x] Password authentication
  - [x] Private key authentication
  - [x] Connection error handling
  - [x] Interactive shell support
  - [x] System SSH client fallback

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
