use bollard::query_parameters::{ListContainersOptions};

use crate::{AppState, error::CommandError, types::Container};

pub async fn list_containers(state : &AppState) -> Result<Vec<Container>, CommandError> {

    let docker = &state.docker;
    let options = ListContainersOptions{
        all: true,
        ..Default::default()
    };
    let res = docker.list_containers(Some(options)).await.map_err(|e| CommandError::DockerError(e.to_string()))?;
    let containers = res.into_iter().map(|item| Container{
        name : item.names.and_then(|names|{
            names.first().map(|name| name.strip_prefix('/').unwrap_or(&name).to_owned())
        }),
        state : item.state,
        status : item.status,
        ports : item.ports.map(|p| p.into_iter().filter_map(|port| port.ip).collect()),
    }).collect();

    Ok(containers)
}