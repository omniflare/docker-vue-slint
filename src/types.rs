use serde::{Serialize, Deserialize};
use bollard::models::ContainerSummaryStateEnum;

#[derive(Debug, Serialize, Deserialize)]
pub struct Container {
    pub name : Option<String>,
    pub status : Option <String>, 
    pub state : Option<ContainerSummaryStateEnum>,
    pub ports : Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Image {
    pub id: String,
    pub repo_tags: Vec<String>,
    pub size : i64
}

