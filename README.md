# container_hosts
Add entries to the hosts-file for each container.

## Building
* `cargo b --release`

## Installing
* `cp ./target/release/container_hosts /usr/local/bin/`
* `cp container_hosts.service /etc/systemd/system/`
* `systemctl daemon-reload && systemctl enable container_hosts`
