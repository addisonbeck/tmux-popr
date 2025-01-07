use clap::Parser;
use std::process::{Command, exit};
use anyhow::{Result, Context, anyhow};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Command to run in the popup (optional)
    #[arg(index = 1)]
    command: Option<String>,
}

struct TmuxSession {
    socket_name: String,
    session_name: String,
}

impl TmuxSession {
    fn new(command: Option<&str>) -> Result<Self> {
        // First check if we're in a tmux session
        if std::env::var("TMUX").is_err() {
            return Err(anyhow!("Not running inside a tmux session"));
        }

        let parent_session = Command::new("tmux")
            .args(["display-message", "-p", "#{session_name}"])
            .output()
            .context("Failed to get parent session name")?;
        
        let parent_session = String::from_utf8_lossy(&parent_session.stdout)
            .trim()
            .to_string();

        // Sanitize the session name to avoid issues with special characters
        let sanitized_session = parent_session
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
            .collect::<String>();

        let suffix = "_popup";
        let prefix = "popup-session";
        
        let session_name = match command {
            Some(cmd) => format!("{}_{}{}", sanitized_session, cmd, suffix),
            None => format!("{}{}", sanitized_session, suffix),
        };
        
        let socket_name = match command {
            Some(cmd) => format!("{}-{}-{}", prefix, sanitized_session, cmd),
            None => format!("{}-{}", prefix, sanitized_session),
        };

        Ok(Self {
            socket_name,
            session_name,
        })
    }

    fn is_attached(&self) -> bool {
        if let Ok(tmux) = std::env::var("TMUX") {
            tmux.contains(&self.socket_name)
        } else {
            false
        }
    }

    fn detach(&self) -> Result<()> {
        Command::new("tmux")
            .args(["-L", &self.socket_name, "detach-client"])
            .status()
            .context("Failed to detach client")?;
        Ok(())
    }

    fn session_exists(&self) -> bool {
        Command::new("tmux")
            .args([
                "-L", &self.socket_name,
                "has-session",
                "-t", &self.session_name,
            ])
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    }

    fn create_session(&self, command: Option<&str>) -> Result<()> {
        let mut cmd = Command::new("tmux");
        cmd.args([
            "-L", &self.socket_name,
            "new-session",
            "-d",
            "-s", &self.session_name,
        ]);

        if let Some(command) = command {
            cmd.arg(command);
        }

        cmd.status()
            .context("Failed to create session")?;

        Command::new("tmux")
            .args([
                "-L", &self.socket_name,
                "set-option",
                "-t", &self.session_name,
                "status", "off",
            ])
            .status()
            .context("Failed to set status off")?;

        Ok(())
    }

    fn show_popup(&self) -> Result<()> {
        let status = Command::new("tmux")
            .args([
                "popup",
                "-d", "#{pane_current_path}",
                "-xC", "-yC",
                "-w95%", "-h95%",
                "-E",
                &format!(
                    "tmux -L {} attach -t \"{}\"",
                    self.socket_name, self.session_name
                ),
            ])
            .status()
            .context("Failed to show popup")?;

        exit(status.code().unwrap_or(1));
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    let session = TmuxSession::new(args.command.as_deref())?;

    if session.is_attached() {
        session.detach()?;
        exit(0);
    }

    if !session.session_exists() {
        session.create_session(args.command.as_deref())?;
    }

    session.show_popup()?;
    Ok(())
}