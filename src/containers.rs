use std::collections::HashMap;

use bollard::models::ContainerCreateBody;
use bollard::query_parameters::{
    KillContainerOptions, RemoveContainerOptions, StopContainerOptions,
};
use bollard::{
    query_parameters::{
        CreateContainerOptions, InspectContainerOptions, ListContainersOptions,
        StartContainerOptions,
    },
    secret::HostConfig,
};

use crate::{AppState, error::CommandError, types::Container};

#[derive(Debug, Clone)]
pub struct ContainerDetails {
    pub id: Option<String>,
    pub name: Option<String>,
    pub image: Option<String>,
    pub created: Option<String>,
    pub state: Option<String>,
    pub status: Option<String>,
    pub networks: Option<Vec<String>>,
    pub ip_addresses: Option<Vec<String>>,
    pub volumes: Option<Vec<String>>,
    pub ports: Option<Vec<String>>,
    pub env: Option<Vec<String>>,
    pub command: Option<String>,
}

pub async fn list_containers(state: &AppState) -> Result<Vec<Container>, CommandError> {
    let docker = &state.docker;
    let options = ListContainersOptions {
        all: true,
        ..Default::default()
    };
    let res = docker
        .list_containers(Some(options))
        .await
        .map_err(|e| CommandError::DockerError(e.to_string()))?;
    let containers = res
        .into_iter()
        .map(|item| Container {
            name: item.names.and_then(|names| {
                names
                    .first()
                    .map(|name| name.strip_prefix('/').unwrap_or(&name).to_owned())
            }),
            state: item.state,
            status: item.status,
            ports: item
                .ports
                .map(|p| p.into_iter().filter_map(|port| port.ip).collect()),
        })
        .collect();

    Ok(containers)
}

pub async fn detail_container_by_id(
    state: &AppState,
    name: &str,
) -> Result<ContainerDetails, CommandError> {
    let docker = &state.docker;
    let res = docker
        .inspect_container(name, None::<InspectContainerOptions>)
        .await
        .map_err(|e| CommandError::DockerError(e.to_string()))?;
    let network_settings = res.network_settings.as_ref();
    let config = res.config.as_ref();
    let networks = network_settings
        .and_then(|n| n.networks.as_ref())
        .map(|n| n.keys().cloned().collect());
    let ip_addresses = network_settings
        .and_then(|n| n.networks.as_ref())
        .map(|nets| nets.values().filter_map(|v| v.ip_address.clone()).collect());

    let ports = network_settings
        .and_then(|n| n.ports.as_ref())
        .map(|p| p.keys().cloned().collect());

    let volumes = res.mounts.as_ref().map(|mounts| {
        mounts
            .iter()
            .map(|m| {
                format!(
                    "{} → {}",
                    m.source.clone().unwrap_or_default(),
                    m.destination.clone().unwrap()
                )
            })
            .collect()
    });

    let env = config.and_then(|c| c.env.clone());
    let command = config
        .and_then(|c| c.cmd.as_ref())
        .map(|cmds| cmds.join(" "));
    let details = ContainerDetails {
        id: res.id,
        name: res.name.map(|n| n.trim_start_matches('/').to_string()),
        image: config.and_then(|c| c.image.clone()),
        created: res.created,
        state: res
            .state
            .as_ref()
            .and_then(|s| s.status.clone())
            .map(|st| st.to_string()),
        status: res
            .state
            .as_ref()
            .and_then(|s| s.status.clone())
            .map(|st| st.to_string()),
        networks,
        ip_addresses,
        volumes,
        ports,
        env,
        command,
    };
    Ok(details)
}

pub async fn create_container(
    state: &AppState,
    image: &str,
    port_mapping: Option<String>,
) -> Result<(), CommandError> {
    let docker = &state.docker;
    // Parse port mapping (e.g. "8080:80")
    let (port_bindings, exposed_ports) = port_mapping
        .as_ref()
        .and_then(|mapping| {
            let parts: Vec<&str> = mapping.split(':').collect();
            if parts.len() != 2 {
                return None;
            }

            let host_port = parts[0];
            let container_port = parts[1];

            let host_binding = vec![bollard::service::PortBinding {
                host_ip: Some("0.0.0.0".into()),
                host_port: Some(host_port.into()),
            }];

            let mut port_bindings = HashMap::new();
            port_bindings.insert(format!("{}/tcp", container_port), Some(host_binding));

            let mut exposed_ports = HashMap::new();
            exposed_ports.insert(format!("{}/tcp", container_port), HashMap::new());

            Some((port_bindings, exposed_ports))
        })
        .unwrap_or_default();

    // Build container config
    let config = ContainerCreateBody {
        image: Some(image.to_string()),
        exposed_ports: if exposed_ports.is_empty() {
            None
        } else {
            Some(exposed_ports)
        },
        host_config: Some(HostConfig {
            port_bindings: if port_bindings.is_empty() {
                None
            } else {
                Some(port_bindings)
            },
            ..Default::default()
        }),
        ..Default::default()
    };

    // Create container
    let res = docker
        .create_container(None::<CreateContainerOptions>, config)
        .await
        .map_err(|e| CommandError::DockerError(format!("Failed to create container: {}", e)))?;

    docker
        .start_container(&res.id, Some(StartContainerOptions::default()))
        .await
        .map_err(|e| CommandError::DockerError(format!("Failed to start container: {}", e)))?;

    Ok(())
}

pub async fn start_container(state: &AppState, name: &str) -> Result<(), CommandError> {
    let docker = &state.docker;
    match docker
        .start_container(name, Some(StartContainerOptions::default()))
        .await
    {
        Ok(_) => {
            println!("Started container '{}' successfully.", name);
            Ok(())
        }
        Err(e) => {
            if e.to_string().contains("No such container") {
                Err(CommandError::DockerError(format!(
                    "Container '{}' not found: {}",
                    name, e
                )))
            } else if e.to_string().contains("permission denied") {
                Err(CommandError::DockerError(format!(
                    "Permission denied while attempting to start container '{}': {}",
                    name, e
                )))
            } else {
                Err(CommandError::DockerError(format!(
                    "Failed to start container '{}': {}",
                    name, e
                )))
            }
        }
    }
}

pub async fn stop_container(state: &AppState, name: &str) -> Result<(), CommandError> {
    let docker = &state.docker;
    match docker
        .stop_container(name, Some(StopContainerOptions::default()))
        .await
    {
        Ok(_) => {
            println!("stopped container with id {}", name);
            Ok(())
        }
        Err(e) => {
            if e.to_string().contains("No such container") {
                Err(CommandError::DockerError(format!(
                    "Container '{}' not found: {}",
                    name, e
                )))
            } else if e.to_string().contains("permission denied") {
                Err(CommandError::DockerError(format!(
                    "Permission denied while attempting to stop container '{}': {}",
                    name, e
                )))
            } else {
                Err(CommandError::DockerError(format!(
                    "Failed to stop container '{}': {}",
                    name, e
                )))
            }
        }
    }
}

pub async fn kill_container(state: &AppState, name: &str) -> Result<(), CommandError> {
    let docker = &state.docker;
    match docker
        .kill_container(name, Some(KillContainerOptions::default()))
        .await
    {
        Ok(_) => {
            println!("killed container with id {}", name);
            Ok(())
        }
        Err(e) => {
            if e.to_string().contains("No such container") {
                Err(CommandError::DockerError(format!(
                    "Container '{}' not found: {}",
                    name, e
                )))
            } else if e.to_string().contains("permission denied") {
                Err(CommandError::DockerError(format!(
                    "Permission denied while attempting to kill container '{}': {}",
                    name, e
                )))
            } else {
                Err(CommandError::DockerError(format!(
                    "Failed to kill container '{}': {}",
                    name, e
                )))
            }
        }
    }
}

pub async fn delete_container(state: &AppState, name: &str) -> Result<(), CommandError> {
    let docker = &state.docker;
    match docker
        .remove_container(name, Some(RemoveContainerOptions::default()))
        .await
    {
        Ok(_) => {
            println!("removed container with id {}", name);
            Ok(())
        }
        Err(e) => {
            if e.to_string().contains("No such container") {
                Err(CommandError::DockerError(format!(
                    "Container '{}' not found: {}",
                    name, e
                )))
            } else if e.to_string().contains("permission denied") {
                Err(CommandError::DockerError(format!(
                    "Permission denied while attempting to kill container '{}': {}",
                    name, e
                )))
            } else {
                Err(CommandError::DockerError(format!(
                    "Failed to kill container '{}': {}",
                    name, e
                )))
            }
        }
    }
}