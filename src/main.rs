mod docker;
mod hosts;
mod hosts_updater;

use clap::Parser;
use hosts_updater::HostsUpdater;
use log::{error, info};
use std::path::PathBuf;
use std::process::exit;
use std::time::Duration;

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

    /// update interval in seconds
    #[arg(long, default_value_t = 30u64)]
    interval: u64,

    /// turn on debug messages
    #[arg(long, default_value_t = false)]
    debug: bool,

    /// exec command on change
    #[arg(long, default_value_t = String::from(""))]
    exec: String,
}

fn main() {
    let args = Args::parse();

    simple_logger::init_with_level(if args.debug {
        log::Level::Debug
    } else {
        log::Level::Info
    })
    .expect("Could not initialize logger");

    let updater = HostsUpdater::new(
        Duration::from_secs(args.interval),
        PathBuf::from(args.hosts),
        &args.docker_socket,
    );

    info!("started...");
    if let Err(e) = updater.update_loop(&args.exec) {
        error!("Error: {:?}", e);
        exit(1);
    }
}
