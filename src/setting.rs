use anyhow::Context;
use std::sync::LazyLock;

pub static SETTING: LazyLock<Setting> =
    LazyLock::new(|| load_setting().expect("cannot load setting"));
const SETTING_FILE_PATH: &str = "./setting.toml";

#[derive(serde::Deserialize, Clone)]
pub struct Setting {
    pub interfaces: Vec<Interface>,
    pub uptime_kuma_url: String,
}

#[derive(serde::Deserialize, Clone)]
pub struct Interface {
    pub name: String,
    pub uptime_api_key: String,
}

pub fn load_setting() -> anyhow::Result<Setting> {
    let file_content = std::fs::read_to_string(SETTING_FILE_PATH)
        .with_context(|| format!("couldn't read setting file located at '{SETTING_FILE_PATH}'"))?;
    let setting: Setting = toml::from_str(&file_content)?;
    Ok(setting)
}
