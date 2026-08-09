//! Web API for the previewer.
//!
//! # API features
//!
//! ## load Gltf: POST /api/load_gltf
//!
//! body: { "gltf": "base64-encoded-gltf" }
//!
//! ## read Gltf info: GET /api/gltf_info
//!
//! returns: {
//!   "status: "loaded",
//!   "scenes" [
//!      {
//!         "animations": [ "idle", "slash", ... ]
//!         "bone_pathes": [
//!             { "root", ["root"] },
//!             { "spine", ["root", "spine"] },
//!             { "head", ["root", "spine", "head"] },
//!             ...
//!         ]
//!      },
//!   ],
//!   "debug_gltf_dump": "..."
//! }
//!
//! ## set animation graph: POST /api/set_anim_graph
//!
//! body: {
//!   // The animation graph description, see `AnimeGraphDesc` in `anim_graph.rs`
//! }
//!
//! ## issue animation graph command: POST /api/anim_graph_command
//!
//! body: {
//!   [ { /* The animation graph command, see `AnimeGraphCommand` in `anim_graph.rs` */ }, ... ]
//! }
//!
//! # Structure
//!
//! For communication between the web API with Bevy app,
//! - API to Bevy : Use MPSC channels to send messages to Bevy app.
//! - Bevy to API : Expose a global state to store the result of Bevy app, and API can read it directly.
//!

use crate::anim_graph;

/// Payload
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub enum ToBevyPayload {
    /// Load Gltf
    LoadGltf { gltf: String },
    /// Set Animation Graph
    SetAnimGraph { anim_graph: anim_graph::AnimeGraphDesc },
    /// Issue Animation Graph Command
    IssueAnimGraphCommand { commands: Vec<anim_graph::AnimeGraphCommand> },
}

use std::sync::Arc;

use axum::{
    Json, Router, debug_handler,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use tokio::sync::{RwLock, mpsc};
use tower_http::cors::Any;
use tower_http::cors::CorsLayer;

use crate::BevyAppExposeState;

#[derive(Clone)]
struct AppState {
    sender: mpsc::Sender<ToBevyPayload>,
    bevy_app_state: Arc<RwLock<BevyAppExposeState>>,
}

pub fn spawn_api_server(
    sender: mpsc::Sender<ToBevyPayload>,
    bevy_app_state: Arc<RwLock<BevyAppExposeState>>,
) {
    let port = std::env::var("API_PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .expect("API_PORT must be a valid u16");

    let state = AppState {
        sender,
        bevy_app_state,
    };

    let app = Router::new()
        .route("/api/load_gltf", post(load_gltf))
        .route("/api/gltf_info", get(get_gltf_info))
        .route("/api/set_anim_graph", post(set_anim_graph))
        .route("/api/anim_graph_command", post(issue_anim_graph_command))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(state);

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to build Tokio runtime");

    runtime.block_on(async move {
        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
            .await
            .expect("Failed to bind API server");
        println!("API server listening on 0.0.0.0:{}", port);
        axum::serve(listener, app)
            .await
            .expect("Failed to run API server");
    });
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
struct LoadGltf {
    gltf: String,
}

impl From<LoadGltf> for ToBevyPayload {
    fn from(payload: LoadGltf) -> Self {
        ToBevyPayload::LoadGltf { gltf: payload.gltf }
    }
}

#[debug_handler]
async fn load_gltf(
    State(state): State<AppState>,
    Json(payload): Json<LoadGltf>,
) -> impl IntoResponse {
    let sender = state.sender.clone();
    tokio::spawn(async move {
        if let Err(e) = sender.send(payload.into()).await {
            eprintln!("Failed to send payload to Bevy: {}", e);
        }
    });
    (StatusCode::OK, "Gltf load request queued")
}

#[debug_handler]
async fn get_gltf_info(State(state): State<AppState>) -> impl IntoResponse {
    let bevy_app_state = state.bevy_app_state.read().await;
    let response = serde_json::json!({
        "status": if bevy_app_state.gltf_path.is_some() { "loaded" } else { "not_loaded" },
        "gltf_path": bevy_app_state.gltf_path,
        "debug_gltf_dump": bevy_app_state.gltf_dump,
        "gltf_info": bevy_app_state.gltf_info,
        "scene_info": bevy_app_state.scene_info,
    });
    (StatusCode::OK, Json(response))
}

#[debug_handler]
async fn set_anim_graph(
    State(state): State<AppState>,
    Json(anim_graph): Json<anim_graph::AnimeGraphDesc>
) -> impl IntoResponse {
    let sender = state.sender.clone();
    tokio::spawn(async move {
        if let Err(e) = sender.send(ToBevyPayload::SetAnimGraph { anim_graph }).await {
            eprintln!("Failed to send payload to Bevy: {}", e);
        }
    });
    (StatusCode::OK, "Animation graph set request queued")
}

#[debug_handler]
async fn issue_anim_graph_command(
    State(state): State<AppState>,
    Json(commands): Json<Vec<anim_graph::AnimeGraphCommand>>
) -> impl IntoResponse {
    let sender = state.sender.clone();
    tokio::spawn(async move {
        if let Err(e) = sender.send(ToBevyPayload::IssueAnimGraphCommand { commands }).await {
            eprintln!("Failed to send payload to Bevy: {}", e);
        }
    });
    (StatusCode::OK, "Animation graph command request queued")
}

