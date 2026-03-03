//! tmux control mode connection
//!
//! Control mode (`tmux -C`) provides a structured, machine-readable interface
//! to tmux. We spawn it as a child process and communicate over stdin/stdout.

use anyhow::{Context, Result};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};

/// A connection to tmux in control mode.
pub struct ControlConnection {
    child: Child,
    reader: Arc<Mutex<BufReader<std::process::ChildStdout>>>,
}

impl ControlConnection {
    /// Attach to an existing tmux session in control mode.
    pub fn attach(session: &str) -> Result<Self> {
        let mut child = Command::new(super::find_tmux())
            .args(["-C", "attach-session", "-t", session])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .context("Failed to spawn tmux in control mode")?;

        let stdout = child
            .stdout
            .take()
            .context("Failed to capture tmux stdout")?;

        let reader = Arc::new(Mutex::new(BufReader::new(stdout)));

        Ok(Self { child, reader })
    }

    /// Send a command string to the control mode session.
    pub fn send_command(&mut self, command: &str) -> Result<()> {
        if let Some(ref mut stdin) = self.child.stdin {
            writeln!(stdin, "{command}")?;
            stdin.flush()?;
            Ok(())
        } else {
            anyhow::bail!("tmux control mode stdin is closed");
        }
    }

    /// Read the next line from control mode output.
    pub fn read_line(&self) -> Result<Option<String>> {
        let mut reader = self.reader.lock().map_err(|e| anyhow::anyhow!("lock: {e}"))?;
        let mut line = String::new();
        let n = reader.read_line(&mut line)?;
        if n == 0 {
            return Ok(None);
        }
        Ok(Some(line.trim_end().to_string()))
    }

    /// Send a command and collect response lines until we see %end or %error.
    pub fn send_and_collect(&mut self, command: &str) -> Result<Vec<String>> {
        self.send_command(command)?;
        let mut lines = Vec::new();
        loop {
            match self.read_line()? {
                Some(line) => {
                    if line.starts_with("%end") || line.starts_with("%error") {
                        break;
                    }
                    if line.starts_with("%begin") {
                        continue;
                    }
                    lines.push(line);
                }
                None => break,
            }
        }
        Ok(lines)
    }

    /// Check if the control mode process is still alive.
    pub fn is_alive(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }

    /// Cleanly shut down the control mode connection.
    pub fn shutdown(&mut self) -> Result<()> {
        let _ = self.send_command("detach");
        std::thread::sleep(std::time::Duration::from_millis(100));
        if self.is_alive() {
            let _ = self.child.kill();
        }
        let _ = self.child.wait();
        Ok(())
    }
}

impl Drop for ControlConnection {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

/// Parse a tmux control mode notification line.
/// Returns (event_type, payload).
pub fn parse_notification(line: &str) -> Option<(&str, &str)> {
    if !line.starts_with('%') {
        return None;
    }
    let first_space = line.find(' ')?;
    Some((&line[..first_space], &line[first_space + 1..]))
}
