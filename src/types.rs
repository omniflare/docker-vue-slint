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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProgressInfo {
    pub status: String,
    pub progress_detail: Option<ProgressDetail>,
    pub id: Option<String>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProgressDetail {
    pub current: Option<i64>,
    pub total: Option<i64>
}