mod curl;
mod ping;

use crate::uptime_kuma;

/// runs set of tests that indicates the wireguard interface is working or not
/// # Returns
/// if wireguard is working right and the necessary tests were successfully it will return
/// `Ok(push_args)`, the `push_args` is filled with some test results and infos
pub async fn do_all_tests(interface_name: &str) -> anyhow::Result<uptime_kuma::PushArgs> {
    let mut messages = Vec::new();
    let mut args = uptime_kuma::PushArgs {
        status: uptime_kuma::Status::Up,
        message: String::new(),
        ping: None,
    };

    let elapsed = curl::curl_some_site(interface_name).await?;
    let elapsed_seconds = elapsed.as_millis() as f32 / 1000.0;
    messages.push(format!("curl OK ({elapsed_seconds:.2}s)"));
    tracing::info!("curl test succeed, took {elapsed_seconds:.2}s");

    match ping::ping_some_host(interface_name).await {
        Ok(ping::PingStatistics {
            packet_lost,
            average_ping,
        }) => {
            args.ping = Some(average_ping);
            messages.push(format!(
                "ping OK ({average_ping}ms, {packet_lost:.2}% lost)"
            ));
            tracing::info!(
                "ping succeed, packet_lost: {packet_lost:.2}, average_ping: {average_ping}",
            )
        }
        Err(error) => {
            // it's ok if ping failed, we care about curl more
            messages.push("ping FAILED".to_owned());
            tracing::warn!("ping failed: {error:?}")
        }
    }

    args.message = messages.join(", ");
    Ok(args)
}
