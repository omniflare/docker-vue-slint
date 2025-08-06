use bollard::container::{
    Config, CreateContainerOptions, KillContainerOptions, LogsOptions, StopContainerOptions,
};
use bollard::query_parameters::ListContainersOptions;

use crate::{AppState, error::CommandError, types::Container};

pub async fn list_containers(state: &AppState) -> Result<Vec<Container>, CommandError> {
    let docker = &state.docker;

    let containers = docker
        .list_containers(Some(ListContainersOptions {
            all: true,
            ..Default::default()
        }))
        .await
        .map_err(|e| CommandError::DockerError(e.to_string()))?;

    let result = containers
        .into_iter()
        .map(|item| Container {
            name: item.names.and_then(|names| {
                names
                    .first()
                    .map(|name| name.strip_prefix('/').unwrap_or(name).to_owned())
            }),
            status: item.status,
            state: item.state,
            ports: item
                .ports
                .map(|ports| ports.into_iter().filter_map(|port| port.ip).collect()),
        })
        .collect();

    Ok(result)
}
