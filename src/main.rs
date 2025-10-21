slint::include_modules!();

mod image;
mod containers;
mod error;
mod types;
use bollard::Docker;

use crate::containers::list_containers;

struct AppState {
    docker: Docker,
}

#[tokio::main]
async fn main() {
    let app = AppWindow::new().unwrap();
    
    let app_state = AppState{
        docker: Docker::connect_with_local_defaults().unwrap(),
    };
    // let containers = list_containers(&app_state).await.unwrap();
    containers::detail_container_by_id(&app_state, "nice_golick").await;
    // println!("{:?}", containers);
    app.run().unwrap();

}
