use anyhow::Result;
use std::time::Duration;

pub struct TelegramNotifier {
    token: String,
    chat_id: String,
    client: reqwest::Client,
}

impl TelegramNotifier {
    pub fn new(token: impl Into<String>, chat_id: impl Into<String>) -> Self {
        Self {
            token: token.into(),
            chat_id: chat_id.into(),
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .expect("Failed to build HTTP client"),
        }
    }

    pub async fn send(&self, message: &str) -> Result<()> {
        let url = format!("https://api.telegram.org/bot{}/sendMessage", self.token);

        let body = serde_json::json!({
            "chat_id": self.chat_id,
            "text": message,
            "parse_mode": "HTML"
        });

        let resp = self.client.post(&url).json(&body).send().await?;

        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Telegram API error: {}", text);
        }

        Ok(())
    }

    pub async fn notify_start(&self, command: &str, pid: u32) -> Result<()> {
        let truncated = truncate_str(command, 500);
        let host = hostname();
        let msg = format!(
            "🟢 <b>agentmode started</b>\n\
             🖥 <b>Host:</b> <code>{}</code>\n\
             💻 <b>Command:</b>\n<code>{}</code>\n\
             🔢 <b>PID:</b> <code>{}</code>",
            escape_html(&host),
            escape_html(&truncated),
            pid
        );
        self.send(&msg).await
    }

    pub async fn notify_done(
        &self,
        command: &str,
        exit_code: i32,
        elapsed_secs: u64,
    ) -> Result<()> {
        let (icon, status) = if exit_code == 0 {
            ("✅", "Success")
        } else {
            ("❌", "Failed")
        };
        let elapsed = format_duration(elapsed_secs);
        let truncated = truncate_str(command, 500);
        let host = hostname();

        let msg = format!(
            "{} <b>agentmode done</b> — <b>{}</b>\n\
             🖥 <b>Host:</b> <code>{}</code>\n\
             💻 <b>Command:</b>\n<code>{}</code>\n\
             🔢 <b>Exit code:</b> <code>{}</code>\n\
             ⏱ <b>Elapsed:</b> {}",
            icon,
            status,
            escape_html(&host),
            escape_html(&truncated),
            exit_code,
            elapsed
        );
        self.send(&msg).await
    }

    #[allow(dead_code)]
    pub async fn notify_error(&self, command: &str, error: &str) -> Result<()> {
        let truncated = truncate_str(command, 300);
        let err_truncated = truncate_str(error, 300);
        let host = hostname();

        let msg = format!(
            "💥 <b>agentmode error</b>\n\
             🖥 <b>Host:</b> <code>{}</code>\n\
             💻 <b>Command:</b>\n<code>{}</code>\n\
             ⚠️ <b>Error:</b> <code>{}</code>",
            escape_html(&host),
            escape_html(&truncated),
            escape_html(&err_truncated),
        );
        self.send(&msg).await
    }
}

/// Returns the machine hostname, or "unknown" on failure.
fn hostname() -> String {
    std::process::Command::new("hostname")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        s.to_string()
    } else {
        let mut truncated: String = s.chars().take(max_len).collect();
        truncated.push_str("...");
        truncated
    }
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn format_duration(secs: u64) -> String {
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_html() {
        assert_eq!(escape_html("foo & bar <baz>"), "foo &amp; bar &lt;baz&gt;");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(45), "45s");
        assert_eq!(format_duration(125), "2m 5s");
        assert_eq!(format_duration(3665), "1h 1m");
    }

    #[test]
    fn test_truncate_str() {
        assert_eq!(truncate_str("hello", 10), "hello");
        assert_eq!(truncate_str("hello world", 5), "hello...");
    }
}
