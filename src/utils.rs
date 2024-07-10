use std::time::Duration;

use anyhow::{bail, Context};
use tokio::{process::Command, time::timeout};

/// # Returns
/// `Ok(stdout_content)` if command exited successfully or `Err(stderr_content)` if command exited
/// with status code other than zero
pub async fn run_command(command: &str, args: &[&str]) -> anyhow::Result<String> {
    tracing::trace!("running command '{command} {}'", args.join(" "));
    let output = Command::new(command)
        .args(args)
        .output()
        .await
        .with_context(|| "couldn't spawn process")?;
    if !output.status.success() {
        let stderr = stdfile_to_string(output.stderr)?;
        bail!("command '{command}' exited with error: {stderr}")
    }

    let stdout = stdfile_to_string(output.stdout)?;
    Ok(stdout)
}

/// # Errors
/// return error if timeout hits or the command exits with an error
pub async fn run_command_with_timeout(
    command: &str,
    args: &[&str],
    timeout_secs: u64,
) -> anyhow::Result<String> {
    tracing::trace!(
        "running command '{command} {}' with timeout {timeout_secs}s",
        args.join(" "),
    );
    let future = Command::new(command).args(args).kill_on_drop(true).output();
    let result = timeout(Duration::from_secs(timeout_secs), future)
        .await
        .map_err(|_| anyhow::anyhow!("{command} timed out"))?;

    let output = result.with_context(|| "couldn't spawn process")?;
    if !output.status.success() {
        let stderr = stdfile_to_string(output.stderr)?;
        bail!("command '{command}' exited with error: {stderr}")
    }

    let stdout = stdfile_to_string(output.stdout)?;
    Ok(stdout)
}

fn stdfile_to_string(bytes: Vec<u8>) -> anyhow::Result<String> {
    let mut contents = String::from_utf8(bytes)?;
    // i don't like the new line at the end of stderr and stdout
    if let Some(last_char) = contents.chars().last() {
        if last_char == '\n' {
            contents.pop().unwrap();
        }
    }
    Ok(contents)
}
