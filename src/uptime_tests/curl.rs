use crate::utils;
use std::time::{Duration, Instant};

const SITE: &str = "https://www.google.com";
const TIMEOUT_SECOND: u64 = 10;

/// # Returns
/// if `Ok` returns the duration that curl took to complete
pub async fn curl_some_site(interface_name: &str) -> anyhow::Result<Duration> {
    let now = Instant::now();
    utils::run_command_with_timeout(
        "curl",
        &[
            "-fsS",
            "--output",
            "/dev/null",
            "--interface",
            interface_name,
            SITE,
        ],
        TIMEOUT_SECOND,
    )
    .await?;
    Ok(now.elapsed())
}
