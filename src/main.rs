mod setting;
mod uptime_kuma;
mod uptime_tests;
mod utils;
mod wireguard;

use anyhow::Context;
use std::{str::FromStr, time::Duration};
use tokio::task::JoinSet;
use tracing::{info_span, Instrument, Level};
use tracing_subscriber::FmtSubscriber;

const SLEEP_TIME_BETWEEN_ROUNDS: Duration = Duration::from_secs(10);

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let log_level = std::env::var("RUST_LOG").unwrap_or(String::from("info"));
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::from_str(&log_level).expect("invalid RUST_LOG"))
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    // ctrl-c exit program immediately
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        std::process::exit(1);
    });

    loop {
        create_all_tasks_and_join_them(&setting::SETTING).await?;
        tokio::time::sleep(SLEEP_TIME_BETWEEN_ROUNDS).await;
    }
}

async fn create_all_tasks_and_join_them(setting: &setting::Setting) -> anyhow::Result<()> {
    let mut tasks = JoinSet::new();

    tracing::info!("spawning tasks...");
    for interface in setting.interfaces.clone() {
        let span = info_span!("report", interface = interface.name);
        tasks.spawn(
            async move {
                let result = create_interface_and_test_it_and_report(&interface).await;
                if let Err(error) = result {
                    tracing::error!("{error:?}");
                }
                // cleaning up interface is important so panic if couldn't clean
                wireguard::delete_interface_if_exists(&interface.name)
                    .await
                    .expect("couldn't cleanup interface after done the tests");
            }
            .instrument(span),
        );
    }

    while let Some(task) = tasks.join_next().await {
        task.with_context(|| "couldn't join task")?;
    }
    tracing::info!("done this round of tests, i'm tired, lets sleep");
    Ok(())
}

async fn create_interface_and_test_it_and_report(
    interface: &setting::Interface,
) -> anyhow::Result<()> {
    wireguard::create_interface(&interface.name)
        .await
        .with_context(|| "failed to create wireguard interface")?;

    let push_args = uptime_tests::do_all_tests(&interface.name).await?;
    tracing::info!("send uptime kuma ok");
    uptime_kuma::push_api(&interface.uptime_api_key, push_args)
        .await
        .with_context(|| "couldn't push api to uptime kuma")?;
    Ok(())
}
