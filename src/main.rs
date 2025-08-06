slint::include_modules!();

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
    let app_state = AppState{
        docker: Docker::connect_with_local_defaults().unwrap(),
    };
    let containers = list_containers(&app_state).await.unwrap();
    println!("{:?}", containers);
    AppWindow::new().unwrap().run().unwrap();
}
