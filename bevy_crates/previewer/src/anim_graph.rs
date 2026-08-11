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

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize, Eq, PartialEq, Hash)]
pub struct MaskGroupIdx(u32);

impl MaskGroupIdx {
    pub fn new(idx: u32) -> Self {
        Self(idx)
    }
    pub fn idx(&self) -> u32 {
        self.0
    }
    pub fn bit(&self) -> u64 {
        1 << self.idx()
    }
}

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MaskGroup {
    /// the bone names for the mask group.
    /// the index `targets` is `MaskGroupIdx` in the `mask` field of nodes.
    /// the pathes of the targets will be included in the Gltf
    pub targets: Vec<String>,
}
#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MaskGroups(pub Vec<MaskGroup>);


// === Commands ===
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum AnimeGraphCommand {
    PlayRepeat(String),
    StopPlay(String),
    SetWeight(String, f32),
}
