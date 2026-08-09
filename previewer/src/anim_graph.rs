use bevy::{platform::collections::HashMap, prelude::*};

// === Graph ===

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AnimeGraphDesc {
    pub nodes: HashMap<String, NodeDesc>,
    pub edges: Vec<EdgeDesc>,
    pub mask_groups: MaskGroups,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum NodeDesc {
    Root,
    Clip(ClipNodeDesc),
    Blend(BlendNodeDesc),
    AdditiveBlend(AdditiveBlendNodeDesc),
}

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ClipNodeDesc {
    pub clip: String,
    pub weight: f32,
    pub mask: Vec<MaskGroupIdx>,
}
#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BlendNodeDesc {
    pub weight: f32,
    pub mask: Vec<MaskGroupIdx>,
}
#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AdditiveBlendNodeDesc {
    pub weight: f32,
    pub mask: Vec<MaskGroupIdx>,
}

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EdgeDesc {
    pub src: String,
    pub dest: String,
}

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MaskGroupIdx(usize);

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MaskGroup {
    /// the bone names for the mask group.
    /// the index `targets` is `MaskGroupIdx` in the `mask` field of nodes.
    /// the pathes of the targets will be included in the Gltf
    targets: Vec<String>,
}
#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MaskGroups(Vec<MaskGroup>);


// === Commands ===
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum AnimeGraphCommand {
    PlayRepeat(String),
    StopPlay(String),
    SetWeight(String, f32),
}
