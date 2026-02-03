use crate::structs::{CreateTask, CreateUser, Db, Task, UpdateTodo, User};
use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};

pub async fn create_user(
    State(db): State<Db>,
    Json(payload): Json<CreateUser>,
) -> impl IntoResponse {
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

pub async fn get_todos(State(db): State<Db>) -> impl IntoResponse {
    let data = db.lock().expect("Locking failed.");

    Json(serde_json::json!({ "tasks": &data.tasks}))
}

pub async fn post_todos(
    State(db): State<Db>,
    Json(payload): Json<CreateTask>,
) -> impl IntoResponse {
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

pub async fn put_todos(
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
            Json(serde_json::json!({"msg": "Task updated", "task": task})),
        );
    }

    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({"msg": "Task not found"})),
    )
}

pub async fn delete_todos(Path(id): Path<usize>, State(db): State<Db>) -> impl IntoResponse {
    let mut data = db.lock().expect("Locking failed.");

    if let Some(pos) = data.tasks.iter().position(|t| t.id == id) {
        let removed_task = data.tasks.remove(pos);
        return (
            StatusCode::OK,
            Json(serde_json::json!({"msg": "Task deleted", "task": removed_task})),
        );
    }
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({"msg": "Task not found"})),
    )
}
