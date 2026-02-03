mod handlers;
mod structs;
mod tokens;

use crate::tokens::{authorize, login};
use axum::{
    Router, middleware,
    routing::{get, patch, post},
};
use handlers::{create_user, delete_todos, get_todos, post_todos, put_todos};
use sqlx::sqlite::SqlitePoolOptions;

#[tokio::main]
async fn main() {
    let db_url = "sqlite://todos.db?mode=rwc";
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(db_url)
        .await
        .expect("Failed to create pool.");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL UNIQUE,
            email TEXT NOT NULL,
            password TEXT NOT NULL
        );",
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS tasks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            completed BOOLEAN NOT NULL DEFAULT 0,
            in_progress BOOLEAN NOT NULL DEFAULT 0,
            user_id INTEGER, -- We will link tasks to users later!
            FOREIGN KEY(user_id) REFERENCES users(id)
        );",
    )
    .execute(&pool)
    .await
    .unwrap();

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
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on port 3000...");
    axum::serve(listener, app).await.unwrap();
}
