use anyhow::Result;
use std::os::unix::process::ExitStatusExt;
use std::process::Stdio;
use tokio::process::{Child, Command};
use tokio::time::{sleep, Duration};

pub struct ManagedProcess {
    child: Child,
    pub pid: u32,
}

impl ManagedProcess {
    pub fn spawn(command: &str) -> Result<Self> {
        // Support shell syntax (pipes, &&, etc.)
        let child = Command::new("sh")
            .arg("-c")
            .arg(command)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;

        let pid = child.id().unwrap_or(0);
        println!("▶ Spawned process PID={}", pid);
        Ok(Self { child, pid })
    }

    /// Wait for process to finish asynchronously without CPU polling.
    /// Returns exit code or signal termination code (128 + signal).
    pub async fn wait_async(&mut self) -> i32 {
        match self.child.wait().await {
            Ok(status) => status
                .code()
                .unwrap_or_else(|| 128 + status.signal().unwrap_or(0)),
            Err(e) => {
                eprintln!("Process wait error: {}", e);
                1
            }
        }
    }
}

impl Drop for ManagedProcess {
    fn drop(&mut self) {
        // Best-effort kill if still running
        let _ = self.child.start_kill();
    }
}

/// Watch an existing PID (attach mode).
pub async fn watch_pid(pid: u32) -> i32 {
    use libc::kill;
    use std::ffi::c_int;

    println!("👁  Watching PID={}", pid);
    loop {
        let res = unsafe { kill(pid as c_int, 0) };
        if res != 0 && std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH) {
            println!("✓ PID={} exited", pid);
            return 0;
        }
        sleep(Duration::from_millis(500)).await;
    }
}
