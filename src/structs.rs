use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

pub type Db = Arc<Mutex<AppState>>;

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateUser {
    pub username: String,
    pub email: String,
    pub password: String,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub id: usize,
    pub username: String,
    pub email: String,
    pub password: String,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct CreateTask {
    pub name: String,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateTodo {
    pub name: Option<String>,
    pub completed: Option<bool>,
    pub in_progress: Option<bool>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Task {
    pub id: usize,
    pub name: String,
    pub completed: bool,
    pub in_progress: bool,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct AppState {
    pub users: Vec<User>,
    pub tasks: Vec<Task>,
}