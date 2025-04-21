use crate::models::session::Session;
use anyhow::Context;
use ssh2::Session as Ssh2Session;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::os::unix::io::AsRawFd;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use termios::{TCSANOW, Termios, tcsetattr};

/// Implementation using the ssh2 crate
pub struct Ssh2Connector;

impl super::SshConnector for Ssh2Connector {
    fn connect(&self, session: &Session) -> anyhow::Result<()> {
        println!(
            "Connecting to {}@{}:{} using ssh2...",
            session.user, session.host, session.port
        );

        // Connect to the remote host
        let tcp = TcpStream::connect((session.host.as_str(), session.port))
            .context("Failed to connect to remote host")?;
        tcp.set_nodelay(true)?;
        tcp.set_read_timeout(Some(Duration::from_secs(10)))?;
        tcp.set_write_timeout(Some(Duration::from_secs(10)))?;

        // Create a new SSH session
        let mut ssh = Ssh2Session::new()?;
        ssh.set_tcp_stream(tcp);
        ssh.handshake()?;

        // Authenticate based on the session's auth type
        match session.auth_type {
            crate::models::session::AuthType::Password => {
                let password = session.password.as_ref().context("Password not found")?;
                ssh.userauth_password(&session.user, password)
                    .context("Password authentication failed")?;
            }
            crate::models::session::AuthType::Key => {
                let key_path = session
                    .private_key_path
                    .as_ref()
                    .context("Private key path not found")?;
                ssh.userauth_pubkey_file(&session.user, None, key_path, None)
                    .context("Key authentication failed")?;
            }
        }

        // Verify authentication was successful
        if !ssh.authenticated() {
            anyhow::bail!("Authentication failed");
        }

        // Create and handle the shell
        create_shell(&mut ssh)?;

        Ok(())
    }
}

/// Creates and handles an interactive shell session
fn create_shell(ssh: &mut Ssh2Session) -> anyhow::Result<()> {
    // Request a shell
    let mut channel = ssh.channel_session()?;

    // Request a pseudo-terminal
    channel.request_pty("xterm-256color", None, Some((80, 24, 0, 0)))?;

    // Request a shell
    channel.shell()?;

    // Set up terminal for raw mode
    let stdin_fd = std::io::stdin().as_raw_fd();
    let mut termios = Termios::from_fd(stdin_fd)?;
    let original_termios = termios;

    // Set raw mode with more permissive settings
    termios.c_iflag &= !(termios::IGNBRK
        | termios::BRKINT
        | termios::PARMRK
        | termios::ISTRIP
        | termios::INLCR
        | termios::IGNCR
        | termios::ICRNL
        | termios::IXON);
    termios.c_oflag &= !termios::OPOST;
    termios.c_lflag &=
        !(termios::ECHO | termios::ECHONL | termios::ICANON | termios::ISIG | termios::IEXTEN);
    termios.c_cflag &= !(termios::CSIZE | termios::PARENB);
    termios.c_cflag |= termios::CS8;
    tcsetattr(stdin_fd, TCSANOW, &termios)?;

    // Create a channel for sending data from stdin to the channel
    let (tx_to_channel, rx_from_stdin) = mpsc::channel::<Vec<u8>>();

    // Spawn a thread to read from stdin and send to the channel
    let tx_to_channel_clone = tx_to_channel.clone();
    thread::spawn(move || {
        let mut stdin = std::io::stdin();
        let mut buffer = [0; 1024];
        loop {
            match stdin.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    if tx_to_channel_clone.send(buffer[..n].to_vec()).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    // Main loop: handle communication between stdin and channel
    let mut buffer = [0; 1024];
    let mut stdout = std::io::stdout();

    loop {
        // Check for data from stdin
        if let Ok(data) = rx_from_stdin.try_recv() {
            if channel.write_all(&data).is_err() {
                break;
            }
        }

        // Read from channel and write directly to stdout
        match channel.read(&mut buffer) {
            Ok(0) => break, // EOF
            Ok(n) => {
                if stdout.write_all(&buffer[..n]).is_err() || stdout.flush().is_err() {
                    break;
                }
            }
            Err(e) => {
                if e.kind() != std::io::ErrorKind::WouldBlock {
                    break;
                }
            }
        }

        // Small sleep to prevent CPU hogging
        thread::sleep(Duration::from_millis(10));
    }

    // Restore terminal settings
    tcsetattr(stdin_fd, TCSANOW, &original_termios)?;

    // Clean up
    channel.close()?;
    ssh.disconnect(None, "Goodbye", None)?;

    Ok(())
}
