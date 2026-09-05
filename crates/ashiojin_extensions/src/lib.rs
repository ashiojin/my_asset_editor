use bevy::{platform::collections::HashMap, prelude::*};

mod common;
mod scene;
mod emitter;
pub mod animation;

pub use scene::SourceGltfHandle as SourceGltfHandle;
pub use scene::AshiojinGltfScene as AshiojinGltfScene;
pub use scene::GltfSceneLabel as GltfSceneLabel;

use crate::common::NotYetExtacted;

#[derive(Debug)]
pub struct AshiojinGltfExtensionsHandlerPlugin;

impl Plugin for AshiojinGltfExtensionsHandlerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SandboxExtension>();
        app.register_type::<SandboxMeshFxConfigExtension>();
        app.register_type::<SandboxActionFxConfig>();
        app.register_type::<SceneArmatureBonePaths>();
        app.register_type::<NotYetExtacted<SandboxActionFxConfig>>();

        app.add_plugins(scene::SceneBasePlugin);
        app.add_plugins(animation::AnimationGraphPlugin::default());
        app.add_plugins(emitter::EmitterPlugin);


        if let Some(mut handlers) = app.world_mut()
            .resource_mut::<bevy::gltf::extensions::GltfExtensionHandlers>()
            .0
            .try_write()
        {
            handlers.push(Box::new(ReplaceMaterialGltfExtensionHandler::default()));
        } else {
            warn!("Failed to acquire write lock for GltfExtensionHandlers");
        }
    }
}


const EXTENSION_NAME_MATERIAL: &str = "ASHIOJIN_material_sandbox";

#[derive(Component, Reflect, Default, serde::Deserialize, serde::Serialize, Debug, Clone)]
#[reflect(Component)]
pub struct SandboxExtension {
    pub shader_type: String,
    pub param1: [f32; 4],
}

const EXTENSION_NAME_MESH_FX_CONFIG: &str = "ASHIOJIN_mesh_fx_config";

#[derive(Component, Reflect, Default, serde::Deserialize, serde::Serialize, Debug, Clone)]
#[reflect(Component)]
pub struct SandboxMeshFxConfigExtension {
    pub is_fx_mesh: bool,
    pub fx_type: String,
}

const EXTENSION_NAME_ACTION_FX_CONFIG: &str = "ASHIOJIN_action_fx_config";

#[derive(Reflect, Default, serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct SandboxActionFxConfigExtension {
    fx_configs: Vec<FxConfig>,
}
#[derive(Reflect, Default, serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct FxConfig {
    pub target_name: String,
    pub start_sec: f32,
    pub end_sec: f32,
}

#[derive(Default, Clone)]
pub struct ReplaceMaterialGltfExtensionHandler {
    // FIXME: rename and/or separate into some handlers
    animation_fx_configs: SandboxActionFxConfig,
}

/// A component that store FxConfig for each animation (action) in the scene.
///
/// It is added to the scene root entity
#[derive(Component, Reflect, Default, Debug, Clone)]
#[reflect(Component)]
#[require(common::NotYetExtacted<SandboxActionFxConfig>)]
pub struct SandboxActionFxConfig {
    pub maps: HashMap<String, Vec<FxConfig>>, // Animation(Action) name -> Vec<FxConfig>
}

const EXTENSION_NAME_SCENE_ARMATURE_BONE_PATHS: &str = "ASHIOJIN_scene_armature_bone_paths";

#[derive(Component, Reflect, Default, serde::Deserialize, serde::Serialize, Debug, Clone)]
#[reflect(Component)]
pub struct SceneArmatureBonePaths{
    pub armature_bone_paths: Vec<ArmatureBonePath>,
}
#[derive(Component, Reflect, Default, serde::Deserialize, serde::Serialize, Debug, Clone)]
#[reflect(Component)]
pub struct ArmatureBonePath {
    pub armature: String,
    /// Bone name -> Bone path from armature root
    pub bone_paths: HashMap<String, Vec<String>>,
}


impl bevy::gltf::extensions::GltfExtensionHandler for ReplaceMaterialGltfExtensionHandler {
    fn dyn_clone(&self) -> Box<dyn bevy::gltf::extensions::ErasedGltfExtensionHandler> {
        Box::new(self.clone())
    }

    fn on_spawn_mesh_and_material(
        &mut self,
        _load_context: &mut bevy::asset::LoadContext<'_>,
        _primitive: &gltf::Primitive,
        mesh: &gltf::Mesh,
        material: &gltf::Material,
        entity: &mut EntityWorldMut,
        _material_label: &str,
    ) {
        if let Some(extension_value) = material.extension_value(EXTENSION_NAME_MATERIAL) {
            let sandbox_extension: SandboxExtension =
                serde_json::from_value(extension_value.clone())
                    .expect("Failed to parse ASHIOJIN_material_sandbox extension");

            if sandbox_extension.shader_type == "ASHIOJIN_SANDBOX" {
                entity.insert(sandbox_extension);

                let t = entity.get_resource::<Assets<StandardMaterial>>().is_some();
                debug!("{t:?}");
            }
        }
        if let Some(extension_value) = mesh.extension_value(EXTENSION_NAME_MESH_FX_CONFIG) {
            let mesh_fx_config_extension: SandboxMeshFxConfigExtension =
                serde_json::from_value(extension_value.clone())
                    .expect("Failed to parse ASHIOJIN_mesh_fx_config extension");
            entity.insert(mesh_fx_config_extension);
        }

        debug!("Mesh {:?}, Ext: {:?}", mesh.name(), mesh.extensions());
        debug!(
            "Material {:?}, Ext: {:?}",
            material.name(),
            material.extensions()
        );
    }

    fn on_animation(
        &mut self,
        _load_context: &mut bevy::asset::LoadContext<'_>,
        gltf_animation: &gltf::Animation,
        _animation_clip: &mut AnimationClip,
    ) {
        if let Some(extension_value) =
            gltf_animation.extension_value(EXTENSION_NAME_ACTION_FX_CONFIG)
        {
            info!(
                "Animation {:?} has extension {:?} = {:?}",
                gltf_animation.name(),
                EXTENSION_NAME_ACTION_FX_CONFIG,
                extension_value
            );
            let fx_config_extenion: SandboxActionFxConfigExtension =
                serde_json::from_value(extension_value.clone())
                    .expect("Failed to parse ASHIOJIN_animation_fx_config extension");
            self.animation_fx_configs.maps.insert(
                gltf_animation.name().unwrap_or_default().to_string(),
                fx_config_extenion.fx_configs,
            );
        }
    }

    fn on_scene_completed(
        &mut self,
        _load_context: &mut bevy::asset::LoadContext<'_>,
        scene: &gltf::Scene,
        world_root_id: Entity,
        scene_world: &mut World,
    ) {
        // add SandboxActionFxConfig component to the root entity of the scene
        info!(
            "Scene {:?} completed, adding SandboxActionFxConfig to root entity {:?}",
            scene.name(),
            world_root_id
        );
        if !self.animation_fx_configs.maps.is_empty() {
            if let Ok(mut root_entity) = scene_world.get_entity_mut(world_root_id) {
                root_entity.insert(SandboxActionFxConfig {
                    maps: self.animation_fx_configs.maps.clone(),
                });
                info!(
                    "Added SandboxActionFxConfig to root entity {:?} with maps: {:?}",
                    world_root_id,
                    self.animation_fx_configs.maps.keys().collect::<Vec<_>>()
                );
            } else {
                warn!("Root entity not found for scene {:?}", scene.name());
            }
        }

        if let Some(scene_armature_bone_paths) = scene.extension_value(EXTENSION_NAME_SCENE_ARMATURE_BONE_PATHS) {
            let scene_armature_bone_paths: SceneArmatureBonePaths =
                serde_json::from_value(scene_armature_bone_paths.clone())
                    .expect("Failed to parse ASHIOJIN_scene_armature_bone_paths extension");
            info!(
                "Scene {:?} has extension {:?} = {:?}",
                scene.name(),
                EXTENSION_NAME_SCENE_ARMATURE_BONE_PATHS,
                scene_armature_bone_paths
            );
            if let Ok(mut root_entity) = scene_world.get_entity_mut(world_root_id) {
                root_entity.insert(scene_armature_bone_paths);
                info!(
                    "Added SceneArmatureBonePaths to root entity {:?}",
                    world_root_id
                );
            } else {
                warn!("Root entity not found for scene {:?}", scene.name());
            }
        }
    }
}
