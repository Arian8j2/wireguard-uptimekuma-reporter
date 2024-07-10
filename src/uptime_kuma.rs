use crate::{setting::SETTING, utils};
use std::fmt::Display;

const TIMEOUT_SECONDS: u64 = 5;

#[allow(unused)]
pub enum Status {
    Up,
    Down,
}

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Self::Up => "up",
            Self::Down => "down",
        };
        write!(f, "{str}")
    }
}

pub struct PushArgs {
    pub status: Status,
    pub message: String,
    pub ping: Option<u32>,
}

// uptimekuma has a monitor type called 'Push' that will report the monitor is up when it gets an
// api call, this function calls that api and tells uptimekuma that the monitor associated with `api_key`
// is up. if uptimekuma doesn't receive push in predefined duration it will report the monitor is down
pub async fn push_api(api_key: &str, args: PushArgs) -> anyhow::Result<()> {
    let uptime_kuma_url: url::Url = SETTING.uptime_kuma_url.parse()?;

    let mut push_api_url = uptime_kuma_url.join("/api/push/").unwrap().join(api_key)?;
    push_api_url.query_pairs_mut().extend_pairs(&[
        ("status", args.status.to_string()),
        ("msg", args.message),
        ("ping", args.ping.map(|x| x.to_string()).unwrap_or_default()),
    ]);

    utils::run_command_with_timeout("curl", &["-fsS", push_api_url.as_str()], TIMEOUT_SECONDS)
        .await?;
    Ok(())
}
