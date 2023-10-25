use crate::docker;
use crate::docker::ListerError;
use crate::hosts::Hosts;

use log::{debug, error, info};
use std::collections::HashSet;
use std::io::Error as IoError;
use std::path::PathBuf;
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

const CONTAINER_SECTION: &str = "DOCKER_CONTAINERS";

#[derive(Debug)]
pub enum HostUpdaterError {
    ListerError(ListerError),
    IoError(IoError),
}

impl From<ListerError> for HostUpdaterError {
    fn from(value: ListerError) -> Self {
        HostUpdaterError::ListerError(value)
    }
}

impl From<IoError> for HostUpdaterError {
    fn from(value: IoError) -> Self {
        HostUpdaterError::IoError(value)
    }
}

pub struct HostsUpdater {
    interval: Duration,
    hosts_file: PathBuf,
    lister: docker::ContainerLister,
}

impl HostsUpdater {
    pub fn new(interval: Duration, hosts_file: PathBuf, docker_socket: &str) -> Self {
        HostsUpdater {
            interval,
            hosts_file,
            lister: docker::ContainerLister::new(docker_socket),
        }
    }

    pub fn update_loop(&self, command: &str) -> Result<(), HostUpdaterError> {
        let mut cmd = Command::new("sh");

        loop {
            sleep(self.interval);
            let daemon_container_entries: HashSet<String> = match self.lister.fetch() {
                Ok(v) => v
                    .iter()
                    .map(|c| format!("{}\t{}", c.addr, c.name))
                    .collect(),
                Err(e) => {
                    error!("Could not fetch containers: {:?}", e);
                    continue;
                }
            };

            debug!("daemon_container_entries: {:?}", &daemon_container_entries);
            let mut hosts = match Hosts::new(self.hosts_file.clone()) {
                Ok(h) => h,
                Err(e) => {
                    error!("Could not open hosts file: {:?}", e);
                    continue;
                }
            };

            let hosts_container_entries = hosts.get_section(Some(CONTAINER_SECTION));

            debug!("hosts_container_entries: {:?}", &hosts_container_entries);

            if (hosts_container_entries
                .as_ref()
                .map(|e| e.len())
                .unwrap_or(0)
                != daemon_container_entries.len())
                || (hosts_container_entries.is_some()
                    && hosts_container_entries
                        .unwrap()
                        .iter()
                        .any(|c| !daemon_container_entries.contains(c)))
            {
                info!(
                    "Container entries of daemon changed, updating {}",
                    self.hosts_file.as_path().display()
                );
                hosts.update_section(
                    Some(CONTAINER_SECTION),
                    daemon_container_entries.iter().cloned(),
                );

                if let Err(e) = hosts.write() {
                    error!(
                        "Could not write to {}: {:?}",
                        self.hosts_file.as_path().display(),
                        e
                    );
                    continue;
                }

                if !command.is_empty() {
                    info!("Executing \"{}\"", command);
                    if let Err(e) = cmd.args(["-c", command]).output() {
                        error!("Failed to execute \"{}\": {:?}", command, e);
                    }
                }
            }
        }
    }
}
