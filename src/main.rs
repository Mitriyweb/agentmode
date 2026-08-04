mod notify;
mod power;
mod process;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::time::Instant;

#[derive(Parser)]
#[command(
    name = "agentmode",
    version = env!("CARGO_PKG_VERSION"),
    about = "Keep your Mac awake while AI agents run — lid closed, no power required",
    long_about = None,
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a command and prevent sleep until it finishes
    Run {
        /// Shell command to run (supports pipes, &&, etc.)
        #[arg(required = true)]
        command: Vec<String>,

        /// Telegram bot token for notifications (or set TELEGRAM_TOKEN env var)
        #[arg(long, env = "TELEGRAM_TOKEN")]
        telegram_token: Option<String>,

        /// Telegram chat ID (or set TELEGRAM_CHAT_ID env var)
        #[arg(long, env = "TELEGRAM_CHAT_ID")]
        telegram_chat_id: Option<String>,
    },

    /// Attach to an existing process by PID and prevent sleep until it exits
    Attach {
        /// PID to watch
        pid: u32,

        /// Telegram bot token for notifications
        #[arg(long, env = "TELEGRAM_TOKEN")]
        telegram_token: Option<String>,

        /// Telegram chat ID
        #[arg(long, env = "TELEGRAM_CHAT_ID")]
        telegram_chat_id: Option<String>,
    },

    /// Just prevent sleep indefinitely (Ctrl+C to stop)
    Keep {
        /// Message to show in Activity Monitor
        #[arg(default_value = "agentmode: keeping awake")]
        reason: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run {
            command,
            telegram_token,
            telegram_chat_id,
        } => {
            let cmd_str = command.join(" ");
            run_command(&cmd_str, telegram_token, telegram_chat_id).await?;
        }

        Commands::Attach {
            pid,
            telegram_token,
            telegram_chat_id,
        } => {
            attach_pid(pid, telegram_token, telegram_chat_id).await?;
        }

        Commands::Keep { reason } => {
            keep_awake(&reason).await?;
        }
    }

    Ok(())
}

async fn run_command(
    cmd: &str,
    telegram_token: Option<String>,
    telegram_chat_id: Option<String>,
) -> Result<()> {
    println!("⚡ agentmode: acquiring power assertion...");
    let _assertion = power::PowerAssertion::acquire()?;
    println!("🔒 Sleep prevented. Mac will stay awake with lid closed.");

    let notifier = build_notifier(telegram_token, telegram_chat_id);

    let mut proc = process::ManagedProcess::spawn(cmd)?;
    let start = Instant::now();

    if let Some(n) = &notifier {
        if let Err(e) = n.notify_start(cmd, proc.pid).await {
            eprintln!("⚠ Telegram error: {}", e);
        }
    }

    println!("─────────────────────────────────────");
    let exit_code = proc.wait_async().await;
    println!("─────────────────────────────────────");

    let elapsed = start.elapsed().as_secs();
    let icon = if exit_code == 0 { "✅" } else { "❌" };
    println!(
        "{} Process finished. Exit={} Elapsed={}s",
        icon, exit_code, elapsed
    );

    if let Some(n) = &notifier {
        if let Err(e) = n.notify_done(cmd, exit_code, elapsed).await {
            eprintln!("⚠ Telegram error: {}", e);
        }
    }

    println!("🔓 Releasing power assertion. Sleep restored.");
    Ok(())
}

async fn attach_pid(
    pid: u32,
    telegram_token: Option<String>,
    telegram_chat_id: Option<String>,
) -> Result<()> {
    println!("⚡ agentmode: acquiring power assertion...");
    let _assertion = power::PowerAssertion::acquire()?;
    println!("🔒 Sleep prevented. Watching PID={}.", pid);

    let notifier = build_notifier(telegram_token, telegram_chat_id);
    let label = format!("PID {}", pid);
    let start = Instant::now();

    if let Some(n) = &notifier {
        if let Err(e) = n.notify_start(&label, pid).await {
            eprintln!("⚠ Telegram error: {}", e);
        }
    }

    let exit_code = process::watch_pid(pid).await;
    let elapsed = start.elapsed().as_secs();

    println!("✅ PID={} is gone. Elapsed={}s", pid, elapsed);

    if let Some(n) = &notifier {
        if let Err(e) = n.notify_done(&label, exit_code, elapsed).await {
            eprintln!("⚠ Telegram error: {}", e);
        }
    }

    println!("🔓 Releasing power assertion. Sleep restored.");
    Ok(())
}

async fn keep_awake(reason: &str) -> Result<()> {
    println!("⚡ agentmode: acquiring power assertion...");
    let _assertion = power::PowerAssertion::acquire()?;
    println!("🔒 Sleep prevented indefinitely. Press Ctrl+C to stop.");
    println!("   Reason: {}", reason);

    // Wait for Ctrl+C
    tokio::signal::ctrl_c().await?;
    println!("\n🔓 Caught Ctrl+C. Releasing assertion. Sleep restored.");
    Ok(())
}

fn build_notifier(
    token: Option<String>,
    chat_id: Option<String>,
) -> Option<notify::TelegramNotifier> {
    match (token, chat_id) {
        (Some(t), Some(c)) if !t.is_empty() && !c.is_empty() => {
            println!("📬 Telegram notifications enabled.");
            Some(notify::TelegramNotifier::new(t, c))
        }
        (Some(t), _) if !t.is_empty() => {
            eprintln!(
                "⚠ TELEGRAM_TOKEN provided without TELEGRAM_CHAT_ID. Notifications disabled."
            );
            None
        }
        (_, Some(c)) if !c.is_empty() => {
            eprintln!(
                "⚠ TELEGRAM_CHAT_ID provided without TELEGRAM_TOKEN. Notifications disabled."
            );
            None
        }
        _ => None,
    }
}
