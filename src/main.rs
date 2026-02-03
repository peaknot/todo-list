mod structs;
mod handlers;

use std::sync::{Arc, Mutex};
use axum::{routing::{get, patch, post}, Router};
use handlers::{create_user, delete_todos, get_todos, post_todos, put_todos};
use structs::{AppState, Db};

#[tokio::main]
async fn main() {
    let db: Db = Arc::new(Mutex::new(AppState {
        users: Vec::new(),
        tasks: Vec::new(),
    }));

    let app: Router = Router::new()
        //.route("/todos", get(get_todos))
        //.route("/todos", post(post_todos))
        .route("/todos", get(get_todos).post(post_todos))
        .route("/todos/{id}", patch(put_todos).delete(delete_todos))
        .route("/users", post(create_user))
        .with_state(db);
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on port 3000...");
    axum::serve(listener, app).await.unwrap();
    
}
