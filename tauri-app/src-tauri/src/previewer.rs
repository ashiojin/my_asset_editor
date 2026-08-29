use std::sync::Arc;

use ashiojin_extensions::animation::{
    AnimationGraphHelper, AnimationGraphSource, LinkToAnimationPlayer,
};
use ashiojin_extensions::{GltfSceneLabel, SceneArmatureBonePaths};
use bevy::winit::WinitPlugin;
use bevy::world_serialization::WorldInstanceReady;
use bevy::{
    animation::AnimationTargetId, gltf::GltfLoaderSettings, platform::collections::HashMap,
    prelude::*,
};
use tauri::async_runtime::Receiver;
use tauri::async_runtime::RwLock;
use tauri::async_runtime::Sender;
use tauri::async_runtime::channel;

use crate::previewer::api::ToPrevewerCommand;

pub mod anim_graph;
pub mod api;

#[derive(Resource, Debug)]
struct FromApi {
    receiver: Receiver<api::ToPrevewerCommand>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub enum SceneSelect {
    ByIndex(usize),
    //ByName(String), // TODO: Implement scene selection api, currently index-0 is always selected
}
impl Default for SceneSelect {
    fn default() -> Self {
        SceneSelect::ByIndex(0)
    }
}

#[derive(Default, Debug, Clone, serde::Serialize)]
pub struct PreviewerState {
    gltf_path: Option<String>,
    gltf_dump: Option<String>,
    gltf_sorted_scene_names: Option<Vec<String>>,
    gltf_scene_select: SceneSelect,

    gltf_info: Option<GltfInfo>,
    scene_info: Option<SceneInfo>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct AnimationInfo {
    name: String,
    idx: usize,
}
#[derive(Default, Debug, Clone, serde::Serialize)]
pub struct GltfInfo {
    animations: Vec<AnimationInfo>,
}
#[derive(Debug, Clone, serde::Serialize)]
pub struct BoneInfo {
    name: String,
    path: Vec<String>,
}
#[derive(Default, Debug, Clone, serde::Serialize)]
pub struct SceneInfo {
    bones: Vec<BoneInfo>,
}

#[derive(Resource, Debug)]
struct BevyAppStateResource {
    state: Arc<RwLock<PreviewerState>>,

    gltf_handle: Option<Handle<Gltf>>,
    is_waiting_gltf_loaded: bool,

    cnt: usize,
}

pub fn run_bevy_app() -> (Sender<ToPrevewerCommand>, Arc<RwLock<PreviewerState>>) {
    let (sender, receiver) = channel::<api::ToPrevewerCommand>(100);
    let bevy_app_state = PreviewerState::default();
    let state = Arc::new(RwLock::new(bevy_app_state));
    let state_cloned = state.clone();
    std::thread::spawn(move || {
        App::new()
            .add_plugins((
                DefaultPlugins
                    .set(AssetPlugin {
                        unapproved_path_mode: bevy::asset::UnapprovedPathMode::Deny,
                        ..default()
                    })
                    .set(WinitPlugin {
                        run_on_any_thread: true,
                    }),
                ashiojin_extensions::AshiojinGltfExtensionsHandlerPlugin,
            ))
            .add_message::<AnimeGraphCommand>()
            .add_message::<DebugCommand>()
            .insert_resource(FromApi { receiver })
            .insert_resource(BevyAppStateResource {
                state: state.clone(),
                gltf_handle: None,
                is_waiting_gltf_loaded: false,
                cnt: 0,
            })
            .add_systems(Startup, setup)
            .add_systems(Update, receive_api_commands)
            .add_systems(Update, spawn_scene_if_gltf_loaded)
            //            .add_systems(Update, apply_anim_graph)
            .add_systems(Update, process_anime_commands)
            .add_systems(Update, debug_print_animation_targets_with_names)
            //           .add_observer(scene_spawned)
            .add_observer(on_scene_ready)
            .run();
    });

    (sender, state_cloned)
}

fn setup(mut commands: Commands) {
    //info!("setup called");
    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // light
    commands.spawn((
        DirectionalLight {
            shadow_maps_enabled: true,
            color: Color::WHITE,
            ..default()
        },
        Transform::from_xyz(0.0, 5.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

#[derive(Component, Debug)]
struct CurrentScene;

#[derive(Message, Debug)]
struct AnimeGraphCommand(anim_graph::AnimeGraphCommand);

#[derive(Message, Debug)]
struct DebugCommand(String);

#[allow(clippy::too_many_arguments)]
fn receive_api_commands(
    mut from_api: ResMut<FromApi>,
    mut bevy_app_state: ResMut<BevyAppStateResource>,
    mut commands: Commands,
    q_current_scene: Query<Entity, With<CurrentScene>>,
    asset_server: Res<AssetServer>,
    mut msgq_anim_graph_command: MessageWriter<AnimeGraphCommand>,
    mut msgq_debug_command: MessageWriter<DebugCommand>,
) {
    //info!("print_for_debug called");
    let receiver = &mut from_api.receiver;
    while let Ok(payload) = receiver.try_recv() {
        info!("Received payload: {:?}", payload);
        match payload {
            api::ToPrevewerCommand::LoadGltf { gltf } => {
                // Remove the current scene if it exists
                for entity in q_current_scene.iter() {
                    commands.entity(entity).try_despawn();
                }
                info!("Loading GLTF: {}", gltf);
                // let handle: Handle<Gltf> = asset_server.load(&gltf);
                let handle: Handle<Gltf> = asset_server
                    .load_builder()
                    .with_settings(|settings: &mut GltfLoaderSettings| {
                        settings.include_source = true
                    })
                    .override_unapproved()
                    .load(&gltf);
                bevy_app_state.gltf_handle = Some(handle);
                bevy_app_state.is_waiting_gltf_loaded = true;

                let mut state = bevy_app_state.state.blocking_write();
                *state = PreviewerState {
                    gltf_path: Some(gltf),
                    ..default()
                };
            }
            api::ToPrevewerCommand::SetAnimGraph { anim_graph } => {
                info!("Setting animation graph: {:?}", anim_graph);
                //anim_graph_desc.set_graph_desc(anim_graph);
                let current_scene_entity = q_current_scene
                    .single()
                    .expect("There should be only one CurrentScene");
                commands
                    .entity(current_scene_entity)
                    .try_insert((AnimationGraphSource::new(anim_graph),));
            }
            api::ToPrevewerCommand::IssueAnimGraphCommand { commands: cmds } => {
                info!("Issuing animation graph commands: {:?}", cmds);
                for cmd in cmds {
                    msgq_anim_graph_command.write(AnimeGraphCommand(cmd));
                }
            }
            api::ToPrevewerCommand::Debug(cmd) => {
                msgq_debug_command.write(DebugCommand(cmd));
            }
        }
    }
}

fn spawn_scene_if_gltf_loaded(
    mut commands: Commands,
    mut bevy_app_state: ResMut<BevyAppStateResource>,
    gltf: Res<Assets<Gltf>>,
) {
    if !bevy_app_state.is_waiting_gltf_loaded {
        return;
    }
    if let Some(handle) = &bevy_app_state.gltf_handle
        && let Some(gltf) = gltf.get(handle)
    {
        let handle_clone = handle.clone();
        bevy_app_state.is_waiting_gltf_loaded = false;
        let mut state = bevy_app_state.state.blocking_write();
        state.gltf_dump = Some(format!("{:?}", gltf.source));
        let scenes = gltf.scenes.iter().cloned();
        let scene_idx_to_name = gltf
            .named_scenes
            .iter()
            .map(|(name, handle)| (handle.clone(), name.to_string()))
            .fold(HashMap::new(), |mut acc, (handle, name)| {
                acc.insert(handle, name);
                acc
            });
        let sorted_scene_names = scenes
            .map(|scene| {
                scene_idx_to_name
                    .get(&scene)
                    .cloned()
                    .unwrap_or_else(|| "Unnamed Scene".to_string())
            })
            .collect();
        state.gltf_sorted_scene_names = Some(sorted_scene_names);
        state.gltf_scene_select = SceneSelect::ByIndex(0);
        let id = commands
            .spawn((
                //WorldAssetRoot(gltf.scenes[0].clone()),
                //SourceGltfHandle(handle_clone),
                ashiojin_extensions::AshiojinGltfScene::new(handle_clone, GltfSceneLabel::Idx(0)),
                Transform::from_xyz(0., 0., 0.),
                CurrentScene,
            ))
            .id();
        info!("Spawned scene entity: {:?}", id);

        let sorted_animations = gltf.animations.iter().cloned();

        let hanime_to_name: HashMap<Handle<AnimationClip>, String> = gltf
            .named_animations
            .iter()
            .map(|(name, handle)| (handle.clone(), name.to_string()))
            .fold(HashMap::new(), |mut acc, (handle, name)| {
                acc.insert(handle, name);
                acc
            });

        let animation_info_v = sorted_animations
            .enumerate()
            .map(|(idx, anim)| AnimationInfo {
                name: hanime_to_name
                    .get(&anim)
                    .cloned()
                    .unwrap_or_else(|| format!("Animation {}", idx)),

                idx,
            })
            .collect::<Vec<_>>();
        let gltf_info = GltfInfo {
            animations: animation_info_v,
        };
        state.gltf_info = Some(gltf_info);
    }
}

fn on_scene_ready(
    scene_ready: On<WorldInstanceReady>,
    q_scene_root: Query<(Entity, &SceneArmatureBonePaths)>,
    q_children: Query<&Children>,
    mut bevy_app_state: ResMut<BevyAppStateResource>,
) {
    let Some((_entity, scene_armature_bone_paths)) = q_children
        .iter_descendants(scene_ready.entity)
        .find_map(|e| q_scene_root.get(e).ok())
    else {
        error!(
            "Not found SceneArmatureBonePaths with {}",
            scene_ready.entity
        );
        return;
    };

    // update state
    let mut bone_info_list = vec![];
    for armature_bone_paths in scene_armature_bone_paths.armature_bone_paths.iter() {
        for (bone_name, bone_path) in armature_bone_paths.bone_paths.iter() {
            let mut path = vec![armature_bone_paths.armature.clone()];
            path.extend_from_slice(bone_path);
            let bone_info = BoneInfo {
                name: bone_name.clone(),
                path,
            };
            bone_info_list.push(bone_info);
        }
    }

    bevy_app_state.state.blocking_write().scene_info = Some(SceneInfo {
        bones: bone_info_list,
    });
    bevy_app_state.cnt = bevy_app_state.cnt.wrapping_add(1);
}
fn process_anime_commands(
    mut msgq_anim_graph_command: MessageReader<AnimeGraphCommand>,
    q_controller: Query<&LinkToAnimationPlayer>,
    mut q_player: Query<(
        &mut AnimationPlayer,
        &AnimationGraphHandle,
        &AnimationGraphHelper,
    )>,
    mut anim_graphs: ResMut<Assets<AnimationGraph>>,
) {
    if msgq_anim_graph_command.is_empty() {
        return;
    }
    let Ok(link_to_player) = q_controller.single() else {
        warn!("No ControlPanel found, cannot process animation graph commands");
        return;
    };
    let Ok((mut player, h_graph, graph_helper)) = q_player.get_mut(link_to_player.player_entity())
    else {
        warn!("No AnimationPlayer found, cannot process animation graph commands");
        return;
    };
    let Some(mut graph) = anim_graphs.get_mut(h_graph) else {
        warn!("No AnimationGraph found, cannot process animation graph commands");
        return;
    };

    for AnimeGraphCommand(cmd) in msgq_anim_graph_command.read() {
        match cmd {
            anim_graph::AnimeGraphCommand::PlayRepeat(node_name) => {
                if let Some(&node_index) = graph_helper.node_id_to_idx().get(node_name.as_str()) {
                    if let Some(anm) = player.animation_mut(node_index) {
                        anm.replay();
                    } else {
                        player.play(node_index).repeat();
                    }
                } else {
                    error!("Node {} not found in ControlPanel", node_name);
                }
            }
            anim_graph::AnimeGraphCommand::StopPlay(node_name) => {
                if let Some(&node_index) = graph_helper.node_id_to_idx().get(node_name.as_str()) {
                    if let Some(_anm) = player.animation_mut(node_index) {
                        player.stop(node_index);
                    } else {
                        info!("Node {} is not playing, cannot stop", node_name);
                    }
                } else {
                    error!("Node {} not found in ControlPanel", node_name);
                }
            }
            anim_graph::AnimeGraphCommand::SetWeight(node_name, weight) => {
                if let Some(&node_index) = graph_helper.node_id_to_idx().get(node_name.as_str()) {
                    let Some(node) = graph.get_mut(node_index) else {
                        error!("Node {} not found in AnimationGraph", node_name);
                        continue;
                    };
                    node.weight = *weight;
                } else {
                    error!("Node {} not found in ControlPanel", node_name);
                }
            }
        }
    }
}

fn debug_print_animation_targets_with_names(
    mut msgq_debug_command: MessageReader<DebugCommand>,
    q_animation_targets: Query<(Entity, &AnimationTargetId, Option<&Name>)>,
) {
    for DebugCommand(cmd) in msgq_debug_command.read() {
        info!("DebugCommand> {}", cmd);
        if cmd == "print_animation_targets" {
            for (e, t, o_n) in q_animation_targets.iter() {
                info!(" {:?}: {:?} -- {:?}", e, t, o_n);
            }
        }
    }
}
