use std::str::from_utf8;

use curl::easy::Easy;
use curl::Error as CurlError;
use serde::{Deserialize, Serialize};

use std::collections::HashMap;

#[derive(Debug)]
pub struct ContainerHostEntry {
    pub name: String,
    pub addr: String,
}

pub struct ContainerLister {
    docker_socket: String,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize, Debug)]
struct ContainerNetwork {
    IPAddress: String,
    Gateway: String,
    IPPrefixLen: u64,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize, Debug)]
struct ContainerNetworks {
    Networks: HashMap<String, ContainerNetwork>,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize, Debug)]
struct Container {
    Id: String,
    Names: Vec<String>,
    NetworkSettings: ContainerNetworks,
}

#[derive(Debug)]
pub enum ListerError {
    CurlError(CurlError),
    ParsingError(serde_json::Error),
}

impl From<CurlError> for ListerError {
    fn from(v: CurlError) -> Self {
        ListerError::CurlError(v)
    }
}

impl From<serde_json::Error> for ListerError {
    fn from(v: serde_json::Error) -> Self {
        ListerError::ParsingError(v)
    }
}

impl ContainerLister {
    pub fn new(docker_socket: &str) -> Self {
        ContainerLister {
            docker_socket: docker_socket.into(),
        }
    }

    pub fn fetch(&self) -> Result<Vec<ContainerHostEntry>, ListerError> {
        let mut c = Easy::new();
        c.unix_socket(&self.docker_socket)?;
        c.url("http://127.0.0.1/containers/json?all=true")?;

        let mut data = String::new();
        {
            let mut trans = c.transfer();
            trans.write_function(|d| {
                if let Ok(s) = from_utf8(d) {
                    data.push_str(s);
                }

                Ok(d.len())
            })?;

            trans.perform()?
        }

        Ok(Self::get_containers(data.as_str())?)
    }

    fn get_containers(data: &str) -> Result<Vec<ContainerHostEntry>, serde_json::Error> {
        let containers: Vec<Container> = serde_json::from_str(data)?;

        let mut hostentries: Vec<ContainerHostEntry> = Vec::new();

        for container in &containers {
            for name in &container.Names {
                for net in container.NetworkSettings.Networks.values() {
                    if !net.IPAddress.is_empty() {
                        hostentries.push(ContainerHostEntry {
                            name: name.to_owned(),
                            addr: net.IPAddress.clone(),
                        })
                    }
                }
            }
        }

        println!("hostentries: {:?}", &hostentries);

        Ok(hostentries)
    }
}
