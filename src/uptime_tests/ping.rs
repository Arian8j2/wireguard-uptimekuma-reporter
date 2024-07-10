use crate::utils;
use regex::Regex;

const HOST: &str = "1.1.1.1";
const INTERVAL: &str = ".5";
const PACKET_COUNT: &str = "100";
const TIMEOUT_SECONDS: u64 = 65;

pub struct PingStatistics {
    pub packet_lost: f32,
    pub average_ping: u32,
}

pub async fn ping_some_host(interface_name: &str) -> anyhow::Result<PingStatistics> {
    let output = utils::run_command_with_timeout(
        "ping",
        &[
            "-I",
            interface_name,
            "-i",
            INTERVAL,
            "-c",
            PACKET_COUNT,
            HOST,
        ],
        TIMEOUT_SECONDS,
    )
    .await?;
    parse_ping_statistics(&output)
}

fn parse_ping_statistics(ping_output: &str) -> anyhow::Result<PingStatistics> {
    // last two lines are the ping statistics, the last two line is like this:
    // 20 packets transmitted, 20 received, 0.302% packet loss, time 5713ms
    // rtt min/avg/max/mdev = 107.564/119.482/140.595/9.421 ms
    let stat_lines = ping_output
        .lines()
        .rev()
        .take(2)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>();

    // '20 packets transmitted, 20 received, 0.302% packet loss, time 5713ms'
    let packet_lost_regex = Regex::new(r", ([\d\.]+)% packet loss").unwrap();
    let packet_lost = packet_lost_regex
        .captures(stat_lines.first().unwrap())
        .ok_or(anyhow::anyhow!(
            "coudn't find packet lost in ping statistics first line"
        ))?
        .get(1)
        .unwrap()
        .as_str();

    // 'rtt min/avg/max/mdev = 107.564/119.482/140.595/9.421 ms'
    let ping_times_regex = Regex::new(r"([\d\.]+\/){3}").unwrap();
    let average_ping = ping_times_regex
        .captures(stat_lines.last().unwrap())
        .ok_or(anyhow::anyhow!(
            "coudn't find ping times in ping statistics second line"
        ))?
        .get(0)
        .unwrap()
        .as_str()
        .split('/')
        .nth(1)
        .unwrap();

    let stats = PingStatistics {
        packet_lost: packet_lost.parse::<f32>().unwrap(),
        average_ping: average_ping.parse::<f32>().unwrap() as u32,
    };
    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ping_statistics() {
        let ping_output = "PING 1.1.1.1 (1.1.1.1) 56(84) bytes of data.
64 bytes from 1.1.1.1: icmp_seq=1 ttl=47 time=123 ms
64 bytes from 1.1.1.1: icmp_seq=2 ttl=47 time=118 ms
64 bytes from 1.1.1.1: icmp_seq=3 ttl=47 time=118 ms
64 bytes from 1.1.1.1: icmp_seq=4 ttl=47 time=125 ms
64 bytes from 1.1.1.1: icmp_seq=5 ttl=47 time=141 ms
64 bytes from 1.1.1.1: icmp_seq=6 ttl=47 time=124 ms
64 bytes from 1.1.1.1: icmp_seq=7 ttl=47 time=119 ms
64 bytes from 1.1.1.1: icmp_seq=8 ttl=47 time=133 ms
64 bytes from 1.1.1.1: icmp_seq=9 ttl=47 time=129 ms
64 bytes from 1.1.1.1: icmp_seq=10 ttl=47 time=117 ms

--- 1.1.1.1 ping statistics ---
10 packets transmitted, 20 received, 0.2332% packet loss, time 5713ms
rtt min/avg/max/mdev = 107.564/119.482/140.595/9.421 ms";

        let stats = parse_ping_statistics(ping_output).unwrap();
        assert_eq!(stats.packet_lost, 0.2332);
        assert_eq!(stats.average_ping, 119);
    }
}
