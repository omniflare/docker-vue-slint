use bollard::{
    models::ContainerCreateBody, query_parameters::{CreateImageOptions, ListImagesOptions, PruneImagesOptions, PushImageOptions, RemoveImageOptions}, secret::HostConfig,
};
use futures_util::StreamExt;
use std::{collections::HashMap, os::unix::process::parent_id, process::Command};

use crate::{error::CommandError, types::{Image, ProgressInfo}, AppState};

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

pub async fn delete_image_by_id(state : &AppState, name : &str) -> Result<(), CommandError>{
    let docker = &state.docker;
    let options = RemoveImageOptions::default();
    let res = docker.remove_image(name, Some(options), None).await.map_err(|e| CommandError::DockerError(e.to_string()));
    Ok(())
}

pub async fn prune_images(state : &AppState) -> Result<(), CommandError> {
    let docker = &state.docker; 
    let options = PruneImagesOptions::default();
    let res = docker.prune_images(Some(options)).await.map_err(|e| CommandError::DockerError(e.to_string()));
    Ok(())
}

pub async fn fetch_from_hub(state : &AppState, name : &str) -> Result<(), CommandError> {
    let docker = &state.docker;
    let options = CreateImageOptions{
        from_image : Some(name.to_string()),
        ..Default::default()
    }; 
    let mut pull_stream = docker.create_image(Some(options), None, None);
    while let Some(res) = pull_stream.next().await {
        match res {
            Ok(output) => {
                 if let Ok(progress) = serde_json::from_value::<ProgressInfo>(
                    serde_json::to_value(output).unwrap_or_default(),
                ){

                }
            }
            Err(e) => {
               return Err(CommandError::DockerError(e.to_string()));
            }
        }
    }
    Ok(())
}

// TODO: Also add option to push data into docker hub.
// Take the config or cred of docker hub from user in app and then push via docker hub. 
