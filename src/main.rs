slint::include_modules!();

mod containers;
mod error;
mod types;
use bollard::Docker;

struct AppState {
    docker: Docker,
}

#[tokio::main]
async fn main() {
    let app = AppWindow::new().unwrap();
    
    let app_state = AppState{
        docker: Docker::connect_with_local_defaults().unwrap(),
    };
    // let containers = list_containers_new_func(&app_state).await.unwrap();

    app.run().unwrap();

}
