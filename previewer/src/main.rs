use std::sync::Arc;

use bevy::{
    gltf::GltfLoaderSettings, platform::collections::HashMap, prelude::*,
    world_serialization::WorldInstanceReady,
};
use tokio::sync::RwLock;

mod anim_graph;
mod api;

#[derive(Resource, Debug)]
struct FromApi {
    receiver: tokio::sync::mpsc::Receiver<api::ToBevyPayload>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub enum SceneSelect {
    ByIndex(usize),
    ByName(String),
}
impl Default for SceneSelect {
    fn default() -> Self {
        SceneSelect::ByIndex(0)
    }
}

#[derive(Default, Debug, Clone, serde::Serialize)]
pub struct BevyAppExposeState {
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
    state: Arc<RwLock<BevyAppExposeState>>,

    gltf_handle: Option<Handle<Gltf>>,
    is_waiting_gltf_loaded: bool,
}

#[derive(Resource, Debug, Default)]
struct AnimeGraphDesc {
    is_applied: bool,
    graph_desc: Option<anim_graph::AnimeGraphDesc>,
}
impl AnimeGraphDesc {
    fn set_graph_desc(&mut self, graph_desc: anim_graph::AnimeGraphDesc) {
        self.graph_desc = Some(graph_desc);
        self.is_applied = false;
    }
}

fn main() {
    let (sender, receiver) = tokio::sync::mpsc::channel::<api::ToBevyPayload>(100);
    let bevy_app_state = BevyAppExposeState::default();
    let state = Arc::new(RwLock::new(bevy_app_state));
    let state_cloned = state.clone();
    std::thread::spawn(move || {
        api::spawn_api_server(sender, state_cloned);
    });
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            unapproved_path_mode: bevy::asset::UnapprovedPathMode::Deny,
            ..default()
        }))
        .add_message::<AnimeGraphCommand>()
        .insert_resource(FromApi { receiver })
        .insert_resource(BevyAppStateResource {
            state: state.clone(),
            gltf_handle: None,
            is_waiting_gltf_loaded: false,
        })
        .insert_resource(AnimeGraphDesc::default())
        .add_systems(Startup, setup)
        .add_systems(Update, receive_api_commands)
        .add_systems(Update, spawn_scene_if_gltf_loaded)
        .add_systems(Update, apply_anim_graph)
        .add_systems(Update, process_anime_commands)
        .add_observer(scene_spawned)
        .run();
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

fn receive_api_commands(
    mut from_api: ResMut<FromApi>,
    mut bevy_app_state: ResMut<BevyAppStateResource>,
    mut anim_graph_desc: ResMut<AnimeGraphDesc>,
    mut commands: Commands,
    q_current_scene: Query<Entity, With<CurrentScene>>,
    asset_server: Res<AssetServer>,
    mut msgq_anim_graph_command: MessageWriter<AnimeGraphCommand>,
) {
    //info!("print_for_debug called");
    let receiver = &mut from_api.receiver;
    while let Ok(payload) = receiver.try_recv() {
        info!("Received payload: {:?}", payload);
        match payload {
            api::ToBevyPayload::LoadGltf { gltf } => {
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
                *state = BevyAppExposeState {
                    gltf_path: Some(gltf),
                    ..default()
                };
            }
            api::ToBevyPayload::SetAnimGraph { anim_graph } => {
                info!("Setting animation graph: {:?}", anim_graph);
                anim_graph_desc.set_graph_desc(anim_graph);
            }
            api::ToBevyPayload::IssueAnimGraphCommand { commands: cmds } => {
                info!("Issuing animation graph commands: {:?}", cmds);
                for cmd in cmds {
                    msgq_anim_graph_command.write(AnimeGraphCommand(cmd));
                }
            }
        }
    }
}

fn spawn_scene_if_gltf_loaded(
    mut commands: Commands,
    // asset_server: Res<AssetServer>,
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
                WorldAssetRoot(gltf.scenes[0].clone()),
                SourceGltfHandle(handle_clone),
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

/// A component that holds the handle to the source GLTF asset.
#[derive(Component, Debug)]
struct SourceGltfHandle(Handle<Gltf>);

#[derive(Component, Debug)]
struct AnimationController {
    animation_player_entity: Entity,
    node_indices: HashMap<String, AnimationNodeIndex>,
}
impl AnimationController {
    fn new(animation_player_entity: Entity) -> Self {
        Self {
            animation_player_entity,
            node_indices: HashMap::new(),
        }
    }
}

fn scene_spawned(
    scene_ready: On<WorldInstanceReady>,
    mut commands: Commands,
    animation_player: Query<Entity, With<AnimationPlayer>>,
    q_children: Query<&Children>,
) {
    info!("Scene : {:?} spawned", scene_ready.entity);
    if let Some(player_entity) = q_children
        .iter_descendants(scene_ready.entity)
        .find_map(|child| animation_player.get(child).ok())
    {
        info!("Found AnimationPlayer entity: {:?}", player_entity);
        commands
            .entity(scene_ready.entity)
            .insert(AnimationController::new(player_entity));
    } else {
        info!("No AnimationPlayer found in the scene");
        info!(
            " animation_players: {:?}",
            animation_player.iter().collect::<Vec<_>>()
        );
    }
}

fn apply_anim_graph(
    mut commands: Commands,
    mut anim_graph_desc: ResMut<AnimeGraphDesc>,
    gltf: Res<Assets<Gltf>>,
    mut q_controller: Query<(Entity, &mut AnimationController, &SourceGltfHandle)>,
    mut animation_graphs: ResMut<Assets<AnimationGraph>>,
) {
    if anim_graph_desc.is_applied {
        return;
    }
    let Some(graph_desc) = &anim_graph_desc.graph_desc else {
        info!("No animation graph description found, it might be cleared, nothing to apply");
        anim_graph_desc.is_applied = true; // No graph to apply. After setting another graph, `is_applied` will be set to false again.
        return;
    };

    let Ok((_entity, mut anim_controller, gltf_handle)) = q_controller.single_mut() else {
        warn!("No AnimController found, cannot apply animation graph");
        return;
    };

    let Some(gltf) = gltf.get(&gltf_handle.0) else {
        error!("GLTF not loaded yet, cannot apply animation graph");
        anim_graph_desc.is_applied = true;
        return;
    };

    // a node except Root node have a edge to parent node, make pairs of (parent_name,node_name, node_desc, is_processed)
    let mut map: Vec<(String, String, anim_graph::NodeDesc, bool)> = Vec::new();
    let mut root_node_name = None;
    for (node_name, node_desc) in &graph_desc.nodes {
        match node_desc {
            anim_graph::NodeDesc::Root => {
                root_node_name = Some(node_name.clone());
            }
            _ => {
                let parent_name = graph_desc
                    .edges
                    .iter()
                    .find(|edge| edge.src == *node_name)
                    .map(|edge| edge.dest.clone());
                if let Some(parent_name) = parent_name {
                    map.push((parent_name, node_name.clone(), node_desc.clone(), false));
                } else {
                    error!("Node {} has no parent, skipping", node_name);
                }
            }
        }
    }

    let Some(root_node_name) = root_node_name else {
        error!("No root node found in the animation graph, cannot apply animation graph");
        anim_graph_desc.is_applied = true;
        return;
    };

    // Make animation graph
    // TODO: Currently ignoring the masks
    let mut graph = AnimationGraph::new();
    let mut node_indices = HashMap::new();
    node_indices.insert(root_node_name.clone(), graph.root);

    while map.iter().any(|(_, _, _, is_processed)| !*is_processed) {
        let mut cnt_processed = 0;
        for (parent_name, node_name, node_desc, is_processed) in map
            .iter_mut()
            .filter(|(_, _, _, is_processed)| !*is_processed)
        {
            let Some(parent_index) = node_indices.get(parent_name).cloned() else {
                // Parent node not processed yet, skip this node for now
                continue;
            };
            *is_processed = true; // Mark this node as processed

            let node_index = match node_desc {
                anim_graph::NodeDesc::Clip(clip_node_desc) => {
                    let Some(h_clip) = gltf.named_animations.get(clip_node_desc.clip.as_str())
                    else {
                        error!("Clip {} not found in GLTF animations", clip_node_desc.clip);
                        continue;
                    };
                    if clip_node_desc.mask.is_empty() {
                        graph.add_clip(h_clip.clone(), clip_node_desc.weight, parent_index)
                    } else {
                        unimplemented!("Masking not implemented yet");
                    }
                }
                anim_graph::NodeDesc::Blend(blend_node_desc) => {
                    if blend_node_desc.mask.is_empty() {
                        graph.add_blend(blend_node_desc.weight, parent_index)
                    } else {
                        unimplemented!("Masking not implemented yet");
                    }
                }
                anim_graph::NodeDesc::AdditiveBlend(additive_blend_node_desc) => {
                    if additive_blend_node_desc.mask.is_empty() {
                        graph.add_additive_blend(additive_blend_node_desc.weight, parent_index)
                    } else {
                        unimplemented!("Masking not implemented yet");
                    }
                }
                anim_graph::NodeDesc::Root => {
                    error!("Root node should not be in the map");
                    continue;
                }
            };
            node_indices.insert(node_name.clone(), node_index);
            cnt_processed += 1;
        }

        if cnt_processed == 0 {
            info!(
                "Could not process any nodes, there might be a cycle in the graph or missing parent nodes"
            );
            break;
        }
    }

    commands
        .entity(anim_controller.animation_player_entity)
        .try_insert(AnimationGraphHandle(animation_graphs.add(graph)));

    anim_controller.node_indices = node_indices;
    anim_graph_desc.is_applied = true;
}

fn process_anime_commands(
    mut msgq_anim_graph_command: MessageReader<AnimeGraphCommand>,
    q_controller: Query<&AnimationController>,
    mut q_player: Query<(&mut AnimationPlayer, &AnimationGraphHandle)>,
    mut anim_graphs: ResMut<Assets<AnimationGraph>>,
) {
    if msgq_anim_graph_command.is_empty() {
        return;
    }
    let Ok(anim_controller) = q_controller.single() else {
        warn!("No ControlPanel found, cannot process animation graph commands");
        return;
    };
    let Ok((mut player, h_graph)) = q_player.get_mut(anim_controller.animation_player_entity) else {
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
                if let Some(&node_index) = anim_controller.node_indices.get(node_name.as_str()) {
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
                if let Some(&node_index) = anim_controller.node_indices.get(node_name.as_str()) {
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
                if let Some(&node_index) = anim_controller.node_indices.get(node_name.as_str()) {
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
