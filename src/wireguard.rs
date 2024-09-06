use super::utils;
use anyhow::Context;
use tokio::fs;

const WIREGUARD_CONFIG_FOLDER: &str = "./wg-configs";

// the commands in here are mostly what 'wg-quick up' would do, the reason i didn't
// used 'wg-quick up' itself is that it also routes all traffic through the created interface and
// we didn't want that here
pub async fn create_interface(name: &str) -> anyhow::Result<()> {
    let config_file_path = format!("{WIREGUARD_CONFIG_FOLDER}/{name}.conf");
    let config = fs::read_to_string(&config_file_path)
        .await
        .with_context(|| format!("couldn't read config file at {config_file_path}"))?;

    utils::run_command("wireguard-go", &[name]).await?;

    // stripped version of common config is needed because `wg setconf` doesn't accept many usual
    // attributes like `MTU`, `DNS`, `Address` and ...
    let stripped_config_path = create_stripped_config(name, &config_file_path).await?;
    utils::run_command("wg", &["setconf", name, &stripped_config_path])
        .await
        .with_context(|| "wg setconf failed")?;

    let maybe_mtu = parse_config_field(&config, "MTU");
    if let Some(mtu) = maybe_mtu {
        utils::run_command("ip", &["link", "set", "mtu", &mtu, "up", "dev", name])
            .await
            .with_context(|| "couldn't set interface mtu")?;
    }

    let address = parse_config_field(&config, "Address")
        .ok_or(anyhow::anyhow!("couldn't find address of config file"))?;
    utils::run_command("ip", &["-4", "address", "add", &address, "dev", name])
        .await
        .with_context(|| "couldn't add address to interface")?;
    Ok(())
}

async fn create_stripped_config(
    interface_name: &str,
    config_file_path: &str,
) -> anyhow::Result<String> {
    let stripped_config = utils::run_command("wg-quick", &["strip", config_file_path])
        .await
        .with_context(|| "couldn't run wg-quick strip")?;

    // `wg setconf` only accepts file path, so we need to write stripped config to a file
    let stripped_config_path = format!("/tmp/{interface_name}.strip");
    tokio::fs::write(&stripped_config_path, stripped_config)
        .await
        .with_context(|| "couldn't write stripped config to temp file")?;
    Ok(stripped_config_path)
}

fn parse_config_field(config: &str, field: &str) -> Option<String> {
    let prefix = format!("{field} = ");
    config.lines().find_map(|line| {
        line.starts_with(&prefix)
            .then_some(line[prefix.len()..].to_owned())
    })
}

pub async fn delete_interface_if_exists(interface_name: &str) -> anyhow::Result<()> {
    utils::run_command("ip", &["link", "delete", "dev", interface_name])
        .await
        .map(|_| ())
        .or_else(|error| {
            if error.to_string().contains("Cannot find device") {
                Ok(())
            } else {
                Err(error)
            }
        })
        .with_context(|| "couldn't delete interface")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_config_field() {
        let config = "[Interface]
Address = 10.88.0.20/16
MTU = 1320
PrivateKey = +BmMvu7A+6VhjaeYELidJjAqkiki8aHhU8QvYoiVCXI=

[Peer]
PublicKey = 0lgrsQFhjRSQ4Tw345d/JKOiUjSoDq2JnbU6dl6/4wl8=
AllowedIPs = 0.0.0.0/0
Endpoint = haha.idk.com:1001";
        assert_eq!(parse_config_field(config, "MTU"), Some("1320".to_owned()));
        assert_eq!(
            parse_config_field(config, "Address"),
            Some("10.88.0.20/16".to_owned())
        );
    }
}
