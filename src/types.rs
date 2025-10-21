use serde::{Serialize, Deserialize};
use bollard::models::ContainerSummaryStateEnum;

#[derive(Debug, Serialize, Deserialize)]
pub struct Container {
    pub name : Option<String>,
    pub status : Option <String>, 
    pub state : Option<ContainerSummaryStateEnum>,
    pub ports : Option<Vec<String>>,
}
