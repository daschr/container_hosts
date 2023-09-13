use std::io::Error as IoError;
use std::process::exit;

use clap::Parser;

mod docker;
mod hosts;
use hosts::Hosts;

/// Generates /etc/hosts entries for all docker containers
#[derive(Parser, Debug)]
#[command(
    version,
    about,
    long_about = None
)]
struct Args {
    /// Unix socket path to docker
    #[arg(long, default_value_t = String::from("/var/run/docker.sock"))]
    docker_socket: String,

    /// Path to hosts
    #[arg(long, default_value_t = String::from("/etc/hosts"))]
    hosts: String,
}

fn main() -> Result<(), IoError> {
    let args = Args::parse();

    let lister = docker::ContainerLister::new(args.docker_socket.as_str());
    let container_entries: Vec<String> = match lister.fetch() {
        Ok(v) => v
            .iter()
            .map(|x| format!("{}\t{}", x.addr, x.name))
            .collect(),
        Err(e) => {
            eprintln!("Could not fetch containers: {:?}", e);
            exit(1);
        }
    };

    if container_entries.len() == 0 {
        exit(0);
    }

    let mut hosts = match Hosts::new(args.hosts) {
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
