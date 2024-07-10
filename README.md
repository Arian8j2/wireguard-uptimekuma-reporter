# Wireguard Uptime-Kuma reporter
monitors multiple wireguard configs and reports them to **[uptime-kuma](https://github.com/louislam/uptime-kuma)**

### How it works?
tests wireguard config files to see if these tests succeed:
- `curl google.com`
- `ping 1.1.1.1`

(curl is treated more important than the ping, so if the ping fails and the curl succeed, it will report status is up but the otherwise is not true)  


and if the tests succeed it will send push api to **uptime-kuma** server, so for this to work you need to first create some monitors with `Push` type

### Usage
create a folder called `reporter` with this schema:
```
reporter
├── setting.toml
└── wg-configs
    ├── idk1.conf
    └── idk2.conf
```
the `setting.toml` is like this:
```
uptime_kuma_url = "http://uptimekuma:3001"

[[interfaces]]
name = "idk1" # name must match the config file name in wg-configs
uptime_api_key = "put_idk1_monitor_push_api_here"

[[interfaces]]
name = "idk2"
uptime_api_key = "put_idk2_monitor_push_api_here"
```
and the `wg-configs` folder contains the actual Wireguard config files  
then use this docker compose to run the service
```
  wireguard-uptimekuma-reporter:
    image: ghcr.io/arian8j2/wireguard-uptimekuma-reporter
    volumes:
      - ./reporter/:/app/
    cap_add:
      - SYS_MODULE
      - NET_ADMIN
    sysctls:
      - net.ipv4.conf.all.src_valid_mark=1
    restart: unless-stopped
```
