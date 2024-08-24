use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Router,
};
use tower_http::trace::TraceLayer;
use uuid::Uuid;

struct AppState {
    store: HashMap<Uuid, String>,
}

#[tokio::main]
async fn main() {
    let state = Arc::new(Mutex::new(AppState {
        store: HashMap::new(),
    }));
    let app = Router::new()
        .route("/store", post(store))
        .route("/load/:uuid", get(load))
        .route("/delete/:uuid", delete(delete_))
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let listner = tokio::net::TcpListener::bind("0.0.0.0:10086")
        .await
        .unwrap();

    axum::serve(listner, app).await.unwrap();
}

async fn store(State(state): State<Arc<Mutex<AppState>>>, body: String) -> impl IntoResponse {
    let uuid = Uuid::new_v4();
    state.lock().unwrap().store.insert(uuid, body);
    uuid.hyphenated().to_string()
}

async fn load(
    State(state): State<Arc<Mutex<AppState>>>,
    Path(uuid): Path<Uuid>,
) -> impl IntoResponse {
    match state.lock().unwrap().store.get(&uuid) {
        Some(text) => text.clone().into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn delete_(
    State(state): State<Arc<Mutex<AppState>>>,
    Path(uuid): Path<Uuid>,
) -> impl IntoResponse {
    match state.lock().unwrap().store.remove(&uuid) {
        Some(_) => ().into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}
