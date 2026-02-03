mod handlers;
mod structs;
mod tokens;

use axum::{
    Router, middleware, routing::{get, patch, post}
};
use handlers::{create_user, delete_todos, get_todos, post_todos, put_todos};
use std::sync::{Arc, Mutex};
use structs::{AppState, Db};

use crate::tokens::{authorize, login};

#[tokio::main]
async fn main() {
    let db: Db = Arc::new(Mutex::new(AppState {
        users: Vec::new(),
        tasks: Vec::new(),
    }));

    let private_routes = Router::new()
            .route("/todos", get(get_todos).post(post_todos))
            .route("/todos/{id}", patch(put_todos).delete(delete_todos))
            .route_layer(middleware::from_fn(authorize));
    
    let public_routes = Router::new()
        .route("/users", post(create_user))
        .route("/login", post(login));

    let app: Router = Router::new()
        //.route("/todos", get(get_todos))
        //.route("/todos", post(post_todos))
        .merge(public_routes)
        .merge(private_routes)
        .with_state(db);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on port 3000...");
    axum::serve(listener, app).await.unwrap();
}
