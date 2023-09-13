use std::env;
use std::io::Error as IoError;
use std::path::Path;
use std::process::exit;

mod docker;
mod hosts;
use hosts::Hosts;

const DOCKER_SOCKET: &str = "/var/run/docker.sock";
const HOSTS: &str = "/etc/hosts";

fn main() -> Result<(), IoError> {
    let lister = docker::ContainerLister::new(DOCKER_SOCKET);
    let container_entries: Vec<String> = match lister.fetch() {
        Ok(v) => v
            .iter()
            .map(|x| format!("{}\t{}", x.name, x.addr))
            .collect(),
        Err(e) => {
            eprintln!("Could not fetch containers: {:?}", e);
            exit(1);
        }
    };

    if container_entries.len() == 0 {
        exit(0);
    }

    let mut hosts = match Hosts::new(HOSTS) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: could not open hosts: {:?}", e);
            exit(1);
        }
    };

    println!("sections: {:?}", hosts.sections);

    hosts.update_section(Some("DOCKER_CONTAINERS"), container_entries);

    hosts.write()?;

    Ok(())
}
