use std::marker::PhantomData;

use bevy::prelude::*;

pub struct SceneBasePlugin;

impl Plugin for SceneBasePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, extract_ashiojin_gltf_scene);
    }
}

/// A component that holds the handle to the source GLTF asset.
#[derive(Component, Debug)]
pub struct SourceGltfHandle(pub Handle<Gltf>);

#[derive(Component, Debug)]
pub enum GltfSceneLabel {
    Name(String),
    Idx(usize),
}

impl GltfSceneLabel {
    pub fn from_name(name: String) -> Self {
        Self::Name(name)
    }

    pub fn from_idx(idx: usize) -> Self {
        Self::Idx(idx)
    }
}

#[derive(Component, Debug)]
#[require(NotYetExtacted<AshiojinGltfScene>)]
pub struct AshiojinGltfScene {
    h_gltf: Handle<Gltf>,
    label: GltfSceneLabel,
}

impl AshiojinGltfScene {
    pub fn new(h_gltf: Handle<Gltf>, scene_label: GltfSceneLabel) -> Self {
        Self {
            h_gltf,
            label: scene_label,
        }
    }
}

#[derive(Component, Debug)]
pub struct NotYetExtacted<T: Component> {
    _dummy: PhantomData<T>,
}
impl<T: Component> Default for NotYetExtacted<T> {
    fn default() -> Self {
        Self { _dummy: default() }
    }
}

/// Extract
pub fn extract_ashiojin_gltf_scene(
    mut commands: Commands,
    q_ashiojin_gltf_scene: Query<
        (Entity, &AshiojinGltfScene),
        With<NotYetExtacted<AshiojinGltfScene>>,
    >,
    gltf: Res<Assets<Gltf>>,
) {
    for (entity, scene) in &q_ashiojin_gltf_scene {
        let Some(gltf) = gltf.get(&scene.h_gltf) else {
            continue;
        };
        let h_scene = match scene.label {
            GltfSceneLabel::Name(ref name) => gltf.named_scenes[name.as_str()].clone(),
            GltfSceneLabel::Idx(idx) => gltf.scenes[idx].clone(),
        };
        commands
            .entity(entity)
            .try_insert((
                SourceGltfHandle(scene.h_gltf.clone()),
                WorldAssetRoot(h_scene),
            ))
            .try_remove::<NotYetExtacted<AshiojinGltfScene>>();
    }
}

