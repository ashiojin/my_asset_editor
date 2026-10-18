use crate::previewer::anim_graph;

/// Payload
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub enum ToPrevewerCommand {
    /// Load Gltf
    LoadGltf { gltf: String },
    /// Set Animation Graph
    SetAnimGraph {
        anim_graph: anim_graph::AnimationGraphDesc,
    },

    /// Issue Animation Graph Command by name
    IssueAnimGraphCommand {
        name: String,
    },

    /// Save Animation Graph
    SaveGraph {
        path: String,
        anim_graph: anim_graph::AnimationGraphDesc,
        pretty: bool,
    },

    /// DebugCommands
    Debug(String),
}
