use bollard::{
    models::ContainerCreateBody, query_parameters::ListImagesOptions, secret::HostConfig,
};
use std::{collections::HashMap, os::unix::process::parent_id};

use crate::{AppState, error::CommandError, types::Image};

#[derive(Debug, Clone)]
pub struct ImageDetails {
    pub id: Option<String>,
    pub parent_id: Option<String>,
    pub repo_tags: Option<Vec<String>>,
    pub created: Option<String>,
    pub size: Option<i64>,
    pub author: Option<String>,
}

pub async fn list_images(state: &AppState) -> Result<Vec<Image>, CommandError> {
    let docker = &state.docker;
    let options = ListImagesOptions {
        all: true,
        ..Default::default()
    };
    let res = docker
        .list_images(Some(options))
        .await
        .map_err(|e| CommandError::DockerError(e.to_string()))?;
    let images = res
        .into_iter()
        .map(|item| Image {
            id: item.id,
            repo_tags: item.repo_tags,
            size: item.size,
        })
        .collect();

    Ok(images)
}

pub async fn detail_image_by_id(
    state: &AppState,
    name: &str,
) -> Result<ImageDetails, CommandError> {
    let docker = &state.docker;
    let res = docker
        .inspect_image(name)
        .await
        .map_err(|e| CommandError::DockerError(e.to_string()))?;
    let details = ImageDetails {
        id: res.id,
        parent_id: Some(parent_id().to_string()),
        repo_tags: res.repo_tags,
        created: res.created,
        size: res.size,
        author: res.author,
    };
    Ok(details)
}
