use crate::docker;
use crate::docker::ListerError;
use crate::hosts::Hosts;

use log::{debug, error, info};
use std::collections::HashMap;
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

#[derive(Debug)]
enum HostsOperationAction {
    Add,
    Del,
    Update,
}

#[derive(Debug)]
struct HostsOperation<'a>(HostsOperationAction, &'a String, &'a String);

impl From<&HostsOperationAction> for &'static str {
    fn from(v: &HostsOperationAction) -> Self {
        match v {
            HostsOperationAction::Add => "add",
            HostsOperationAction::Del => "del",
            HostsOperationAction::Update => "update",
        }
    }
}

impl HostsUpdater {
    pub fn new(interval: Duration, hosts_file: PathBuf, docker_socket: &str) -> Self {
        HostsUpdater {
            interval,
            hosts_file,
            lister: docker::ContainerLister::new(docker_socket),
        }
    }

    pub fn update_loop(&self, command: Option<String>) -> Result<(), HostUpdaterError> {
        let mut cmd = Command::new("sh");

        loop {
            sleep(self.interval);
            let daemon_container_entries: Vec<(String, String)> = match self.lister.fetch() {
                Ok(v) => v.iter().map(|c| (c.name.clone(), c.addr.clone())).collect(),
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

            let mut hosts_container_entries: HashMap<String, String> = HashMap::new();
            if let Some(hc_entries) = hosts.get_section(Some(CONTAINER_SECTION)) {
                for entry in hc_entries {
                    let entry = entry.trim();
                    let mut spl = entry.split('\t');
                    if let (Some(name), Some(addr)) = (spl.next(), spl.next()) {
                        hosts_container_entries.insert(name.to_string(), addr.to_string());
                    }
                }
            }

            debug!("hosts_container_entries: {:?}", &hosts_container_entries);

            let mut operations: Vec<HostsOperation> = Vec::new();

            for e in daemon_container_entries.iter() {
                if hosts_container_entries.contains_key(&e.0) {
                    if hosts_container_entries.remove(&e.0).unwrap() != e.1 {
                        operations.push(HostsOperation(HostsOperationAction::Update, &e.0, &e.1));
                    }
                } else {
                    operations.push(HostsOperation(HostsOperationAction::Add, &e.0, &e.1));
                }
            }

            for e in hosts_container_entries.iter() {
                operations.push(HostsOperation(HostsOperationAction::Del, e.0, e.1));
            }

            debug!("operations: {:?}", &operations);

            if !operations.is_empty() {
                info!(
                    "Container entries of daemon changed, updating {}",
                    self.hosts_file.as_path().display()
                );
                hosts.update_section(
                    Some(CONTAINER_SECTION),
                    daemon_container_entries
                        .iter()
                        .map(|c| format!("{}\t{}", c.0, c.1)),
                );

                if let Err(e) = hosts.write() {
                    error!(
                        "Could not write to {}: {:?}",
                        self.hosts_file.as_path().display(),
                        e
                    );
                    continue;
                }

                if let Some(command) = command.as_ref() {
                    info!("Executing \"{}\"", command);

                    for op in operations {
                        let cmd_args = ["-c", command];

                        match cmd
                            .args(cmd_args)
                            .env("HOSTS_ACTION", Into::<&str>::into(&op.0))
                            .env("HOSTS_NAME", op.1)
                            .env("HOSTS_ADDR", op.2)
                            .output()
                        {
                            Ok(c) => debug!("Command executed: {:?}", c),
                            Err(e) => error!("Failed to execute \"{}\": {:?}", command, e),
                        }
                    }
                }
            }
        }
    }
}
