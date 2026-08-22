use std::sync::Arc;

use tauri::{
    Manager, State, async_runtime::{RwLock, Sender},
};

use crate::previewer::{PreviewerState, api::ToPrevewerCommand};

mod previewer;
// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// FIXME: Define & Use errors
#[tauri::command]
async fn load_gltf(state: State<'_, AppState>, path: &str) -> Result<(), ()> {
    state.sender.send(
        ToPrevewerCommand::LoadGltf { gltf: path.to_string() }
    ).await;
    Ok(())
}

// FIXME: Define & Use errors
#[tauri::command]
async fn get_state(state: State<'_, AppState>) -> Result<PreviewerState, ()> {
    let s = state.bevy_app_state.read().await;

    Ok(s.to_owned())
}

// FIXME: Define & Use errors
#[tauri::command]
async fn set_graph(state: State<'_, AppState>, graph: previewer::anim_graph::AnimeGraphDesc) -> Result<(), ()> {
    state.sender.send(
        ToPrevewerCommand::SetAnimGraph { anim_graph: graph })
        .await;
    Ok(())
}

// FIXME: Define & Use errors
#[tauri::command]
async fn issue_graph_commands(state: State<'_, AppState>, commands: Vec<previewer::anim_graph::AnimeGraphCommand>) -> Result<(), ()> {
    state.sender.send(
        ToPrevewerCommand::IssueAnimGraphCommand { commands })
        .await;
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
            let (sender, previewer_state) = previewer::run_bevy_app();
            app.manage(AppState {
                sender,
                bevy_app_state: previewer_state,
            });
            Ok(())
        })
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
