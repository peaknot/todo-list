use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, patch},
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

type Db = Arc<Mutex<AppState>>;

#[derive(Serialize, Deserialize, Debug)]
struct CreateUser {
    username: String,
    email: String,
    password: String,
}
#[derive(Serialize, Deserialize, Debug)]
struct User {
    id: usize,
    username: String,
    email: String,
    password: String,
}
#[derive(Serialize, Deserialize, Debug)]
struct CreateTask {
    name: String,
}
#[derive(Serialize, Deserialize, Debug)]
struct UpdateTodo {
    name: Option<String>,
    completed: Option<bool>,
    in_progress: Option<bool>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
struct Task {
    id: usize,
    name: String,
    completed: bool,
    in_progress: bool,
}
#[derive(Serialize, Deserialize, Debug)]
struct AppState {
    users: Vec<User>,
    tasks: Vec<Task>,
}

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

async fn create_user(State(db): State<Db>, Json(payload): Json<CreateUser>) -> impl IntoResponse {
    let mut data = db.lock().unwrap();

    let new_id = data.users.iter().map(|x| x.id + 1).max().unwrap_or(1);

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(payload.password.as_bytes(), &salt)
        .expect("Password hashing failed")
        .to_string();

    let new_user = User {
        id: new_id,
        username: payload.username,
        email: payload.email,
        password: password_hash,
    };
    data.users.push(new_user);

    (
        StatusCode::CREATED,
        Json(serde_json::json!({"msg": "User created", "id": new_id})),
    )
}

async fn get_todos(State(db): State<Db>) -> impl IntoResponse {
    let data = db.lock().expect("Locking failed.");

    Json(serde_json::json!({ "tasks": &data.tasks}))
}

async fn post_todos(State(db): State<Db>, Json(payload): Json<CreateTask>) -> impl IntoResponse {
    let mut data = db.lock().expect("Locking failed.");

    let task_id = data.tasks.iter().map(|n| n.id + 1).max().unwrap_or(1);

    let new_task = Task {
        id: task_id,
        name: payload.name,
        completed: false,
        in_progress: false,
    };
    data.tasks.push(new_task);
    (
        StatusCode::CREATED,
        Json(serde_json::json!({"msg": "Task created", "id": task_id})),
    )
}

async fn put_todos(
    Path(id): Path<usize>,
    State(db): State<Db>,
    Json(input): Json<UpdateTodo>,
) -> impl IntoResponse {
    let mut data = db.lock().expect("Locking failed.");

    if let Some(task) = data.tasks.iter_mut().find(|t| t.id == id) {
        if let Some(new_name) = input.name {
            task.name = new_name;
        }
        if let Some(is_completed) = input.completed {
            task.completed = is_completed;
            if is_completed {
                task.in_progress = false;
            }
        }
        if let Some(is_in_progress) = input.in_progress {
            task.in_progress = is_in_progress;
            if is_in_progress {
                task.completed = false;
            }
        }

        return (
            StatusCode::OK,
            Json(serde_json::json!({"msg": "Task updated", "task": task}))
        );
    }

    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({"msg": "Task not found"}))
    )
}

async fn delete_todos(Path(id): Path<usize>, State(db): State<Db>) -> impl IntoResponse {
    let mut data = db.lock().expect("Locking failed.");

    if let Some(pos) = data.tasks.iter().position(|t| t.id == id) {
        let removed_task = data.tasks.remove(pos);
        return (
            StatusCode::OK,
            Json(serde_json::json!({"msg": "Task deleted", "task": removed_task}))
        );
    }
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({"msg": "Task not found"}))
    )
}
