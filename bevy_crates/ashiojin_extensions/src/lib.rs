use bevy::{platform::collections::HashMap, prelude::*};

#[derive(Debug)]
pub struct AshiojinGltfExtensionsHandlerPlugin;

impl Plugin for AshiojinGltfExtensionsHandlerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SandboxExtension>();
        app.register_type::<SandboxMeshFxConfigExtension>();
        app.register_type::<SandboxActionFxConfig>();
        app.register_type::<SceneArmatureBonePaths>();

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

#[derive(Component, Reflect, Default, Debug, Clone)]
#[reflect(Component)]
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

// pub fn apply_sandbox_fx_meshes(
//     mut commands: Commands,
//     _time: Res<Time>,
//     #[allow(clippy::type_complexity)] query: Query<
//         (
//             Entity,
//             &SandboxMeshFxConfigExtension,
//             &Name,
//             Option<&Mesh3d>,
//         ),
//         With<SandboxMeshFxConfigExtension>,
//     >,
//     q_scene_root: Query<(Entity, &AutoPlay), Added<AutoPlay>>,
//     q_fx_cocnfig: Query<(Entity, &SandboxActionFxConfig)>,
//     q_children: Query<&Children>,
// ) {
//     // Add TrailEmitter
//     for (root_entity, auto_play) in q_scene_root.iter() {
//         let Some((_, fx_config)) = q_children
//             .iter_descendants(root_entity)
//             .find_map(|entity| q_fx_cocnfig.get(entity).ok())
//         else {
//             info!(
//                 "Scene root entity {:?} has AutoPlay but no SandboxActionFxConfig",
//                 root_entity
//             );
//             continue;
//         };
//
//         info!(
//             "Scene root entity {:?} has AutoPlay and SandboxActionFxConfig with maps: {:?}",
//             root_entity,
//             fx_config.maps.keys().collect::<Vec<_>>()
//         );
//         q_children.iter_descendants(root_entity).for_each(|entity| {
//             if let Ok((_, mesh_fx_config_extension, name, _mesh3d)) =
//                 query.get(entity)
//             {
//                 if !mesh_fx_config_extension.is_fx_mesh {
//                     info!(
//                         "Skipping entity {:?} (name: {:?}) because is_fx_mesh is false",
//                         entity,
//                         name.as_str()
//                     );
//                     return;
//                 }
//                 // TODO: Should use `fx_type` to determine which effect to apply. For now, we only have one effect, so we ignore it.
//                 // The below code assumes that the `fx_type` is always "trail", and we will add a TrailEmitter to the entity.
//
//                 let mut timings = vec![];
//                 for (action_name, fx_configs) in fx_config.maps.iter() {
//                     let Some(anim_node_idx) = auto_play
//                         .node_idx_list()
//                         .iter()
//                         .find(|(n, _)| n.as_str() == action_name.as_str())
//                         .map(|(_, idx)| idx)
//                     else {
//                         continue;
//                     };
//                     let fx_configs = fx_configs
//                         .iter()
//                         .filter(|fx| fx.target_name == name.as_str())
//                         .map(|fx| TrailEmitterTiming::new(*anim_node_idx, fx.start_sec, fx.end_sec));
//
//                     timings.extend(fx_configs);
//                 }
//
//                 info!(
//                     "-- node_idx_list: {:?}, fx_config: {:?}, entity: {:?}, name: {:?}, timings: {:?}",
//                     auto_play.node_idx_list(),
//                     fx_config,
//                     entity,
//                     name.as_str(),
//                     timings
//                 );
//
//                 info!(
//                     "Adding TrailEmitter to entity {:?} (name: {:?}) with timings: {:?}",
//                     entity,
//                     name.as_str(),
//                     timings
//                 );
//
//                 commands.entity(entity).try_insert((
//                     TrailEmitter::new(0.2).extend_timings(timings),
//                     Visibility::Hidden,
//                 ));
//             }
//         });
//     }
// }
