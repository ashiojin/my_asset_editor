use bevy::prelude::*;

// === Graph ===
pub use ashiojin_extensions::animation::graph_desc::*;

// === Commands ===
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum AnimeGraphCommand {
    PlayRepeat(String),
    StopPlay(String),
    SetWeight(String, f32),
}
