use crate::previewer::anim_graph;

/// Payload
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub enum ToPrevewerCommand {
    /// Load Gltf
    LoadGltf { gltf: String },
    /// Set Animation Graph
    SetAnimGraph {
        anim_graph: anim_graph::AnimeGraphDesc,
    },
    /// Issue Animation Graph Command
    IssueAnimGraphCommand {
        commands: Vec<anim_graph::AnimeGraphCommand>,
    },

    /// DebugCommands
    Debug(String),
}
