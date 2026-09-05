use std::sync::Arc;

use tauri::{
    Manager, State,
    async_runtime::{RwLock, Sender},
};

use crate::previewer::{PreviewerState, api::ToPrevewerCommand};

mod previewer;
// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}
#[derive(thiserror::Error, Debug, serde::Serialize)]
struct SendError {
    message: String,
}
impl SendError {
    fn new(message: String) -> Self {
        Self { message }
    }
}
impl std::fmt::Display for SendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[derive(thiserror::Error, Debug, serde::Serialize)]
enum ErrorLoadGltf {
    #[error("SendError: {0}")]
    SendError(#[from] SendError),
    // #[error("Unknonw error")]
    // Unknown,
}

#[tauri::command]
async fn load_gltf(state: State<'_, AppState>, path: &str) -> Result<(), ErrorLoadGltf> {
    state
        .sender
        .send(ToPrevewerCommand::LoadGltf {
            gltf: path.to_string(),
        })
        .await
        .map_err(|send_err| SendError::new(send_err.to_string()))?;
    Ok(())
}

#[tauri::command]
async fn get_state(state: State<'_, AppState>) -> Result<PreviewerState, ()> {
    let s = state.bevy_app_state.read().await;

    Ok(s.to_owned())
}

#[derive(thiserror::Error, Debug, serde::Serialize)]
enum ErrorSetGraph {
    #[error("SendError: {0}")]
    SendError(#[from] SendError),
}

#[tauri::command]
async fn set_graph(
    state: State<'_, AppState>,
    graph: previewer::anim_graph::AnimationGraphDesc,
) -> Result<(), ErrorSetGraph> {
    state
        .sender
        .send(ToPrevewerCommand::SetAnimGraph { anim_graph: graph })
        .await
        .map_err(|e| SendError::new(e.to_string()))?;
    Ok(())
}

#[derive(thiserror::Error, Debug, serde::Serialize)]
enum ErrorIssueGraphCommands {
    #[error("SendError: {0}")]
    SendError(#[from] SendError),
}

#[tauri::command]
async fn issue_graph_commands(
    state: State<'_, AppState>,
    commands: Vec<previewer::anim_graph::AnimeGraphCommand>,
) -> Result<(), ErrorIssueGraphCommands> {
    state
        .sender
        .send(ToPrevewerCommand::IssueAnimGraphCommand { commands })
        .await
        .map_err(|e| SendError::new(e.to_string()))?;
    Ok(())
}

#[derive(Clone)]
struct AppState {
    sender: Sender<ToPrevewerCommand>,
    bevy_app_state: Arc<RwLock<PreviewerState>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(move |app| {
            let (sender, previewer_state) = previewer::run_bevy_app(app.handle().clone());
            app.manage(AppState {
                sender,
                bevy_app_state: previewer_state,
            });
            Ok(())
        })
        // .on_window_event(|window, event| {
        //     if let tauri::WindowEvent::CloseRequested { api, .. } = event {
        //         // Send a command to the previewer to close itself
        //         let app_state: State<AppState> = window.state();
        //         let sender = app_state.sender.clone();
        //         tauri::async_runtime::spawn(async move {
        //             let _ = sender.send(ToPrevewerCommand::ClosePreviewer).await;
        //         });
        //     }
        // })
        .invoke_handler(tauri::generate_handler![
            greet,
            load_gltf,
            get_state,
            set_graph,
            issue_graph_commands,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
