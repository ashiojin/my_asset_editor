use bevy::{
    animation::{AnimationTargetId, RepeatAnimation},
    platform::collections::HashMap,
    prelude::*,
    world_serialization::WorldInstanceReady,
};

use crate::{
    SceneArmatureBonePaths, SourceGltfHandle, animation::{event::InnerClipEvent, graph_desc::{AnimationGraphDesc, command_desc::HasCommandTarget}}, common::NotYetExtacted,
};

pub mod graph_desc;

mod command;
mod event;

pub use command::{AnimationGraphCommandRequest};
pub use event::ClipNodeEvent;

#[derive(Default, Debug)]
pub struct AnimationGraphPlugin {
    _debug_mode: bool, // TODO: add systems that check invariants when debug_mode is true
}

impl Plugin for AnimationGraphPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(scene_spawned);

        app.add_systems(
            Update,
            (
                apply_anim_graph,
                // command::apply_anim_graph_commands_desc,
            ),
        );
    }
}

#[derive(Component, Debug)]
pub struct LinkToAnimationPlayer(Entity);

impl LinkToAnimationPlayer {
    pub fn player_entity(&self) -> Entity {
        self.0
    }
}

#[derive(Component, Debug)]
pub struct AnimationGraphHelper {
    /// Name of node -> its idx
    node_id_to_idx: HashMap<String, AnimationNodeIndex>,

    node_idx_to_clip_handle: HashMap<AnimationNodeIndex, Handle<AnimationClip>>,

    /// Clip name -> node idx
    clip_name_to_node_idx_list: HashMap<String, Vec<AnimationNodeIndex>>,

    /// Clip name -> handle
    clip_name_to_handle: HashMap<String, Handle<AnimationClip>>,
}

/// Helper to store some useful information about the animation graph
///
/// `AnimationGraphHelper` is inserted into the `AnimationPlayer` entity, not the scene root entity
/// nor the `AnimationGraphSource` entity.
impl AnimationGraphHelper {
    pub fn new(
        node_id_to_idx: HashMap<String, AnimationNodeIndex>,
        clip_name_to_node_idx_list: HashMap<String, Vec<AnimationNodeIndex>>,
        clip_name_to_handle: HashMap<String, Handle<AnimationClip>>,
    ) -> Self {
        let mut node_idx_to_clip_handle = HashMap::new();
        for (clip_name, node_idx_list) in &clip_name_to_node_idx_list {
            if let Some(node_idx) = node_idx_list.first()
                && let Some(handle) = clip_name_to_handle.get(clip_name)
            {
                node_idx_to_clip_handle.insert(*node_idx, handle.clone());
            }
        }
        Self {
            node_id_to_idx,
            node_idx_to_clip_handle,
            clip_name_to_node_idx_list,
            clip_name_to_handle,
        }
    }

    pub fn node_id_to_idx(&self) -> &HashMap<String, AnimationNodeIndex> {
        &self.node_id_to_idx
    }

    pub fn node_idx_to_clip_handle(&self) -> &HashMap<AnimationNodeIndex, Handle<AnimationClip>> {
        &self.node_idx_to_clip_handle
    }

    pub fn clip_name_to_node_idx(&self) -> &HashMap<String, Vec<AnimationNodeIndex>> {
        &self.clip_name_to_node_idx_list
    }

    pub fn clip_name_to_handle(&self) -> &HashMap<String, Handle<AnimationClip>> {
        &self.clip_name_to_handle
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
            .insert(LinkToAnimationPlayer(player_entity));
    } else {
        info!("No AnimationPlayer found in the scene");
        info!(
            " animation_players: {:?}",
            animation_player.iter().collect::<Vec<_>>()
        );
    }
}

#[derive(Component, Debug)]
#[require(NotYetExtacted<AnimationGraphSource>)]
pub struct AnimationGraphSource(AnimationGraphDesc);

impl AnimationGraphSource {
    pub fn new(desc: AnimationGraphDesc) -> Self {
        Self(desc)
    }
}

fn apply_anim_graph(
    mut commands: Commands,
    //mut anim_graph_desc: ResMut<AnimeGraphDesc>,
    gltf: Res<Assets<Gltf>>,
    q_target: Query<
        (
            Entity,
            &AnimationGraphSource,
            &LinkToAnimationPlayer,
            &SourceGltfHandle,
        ),
        With<NotYetExtacted<AnimationGraphSource>>,
    >,
    q_scene_root: Query<(Entity, &SceneArmatureBonePaths)>,
    q_children: Query<&Children>,
    mut animation_graphs: ResMut<Assets<AnimationGraph>>,
    mut animatnion_clips: ResMut<Assets<AnimationClip>>,
) {
    use graph_desc::*;

    for (entity, AnimationGraphSource(graph_desc), link_to_player, SourceGltfHandle(gltf_handle)) in
        &q_target
    {
        info!("{:?}", entity);
        let Some(gltf) = gltf.get(gltf_handle) else {
            error!("GLTF not loaded yet, cannot apply animation graph");
            commands
                .entity(entity)
                .try_remove::<NotYetExtacted<AnimationGraphSource>>();
            return;
        };

        // a node except Root node have a edge to parent node, make pairs of (parent_name,node_name, node_desc, is_processed)
        let mut map: Vec<(String, String, NodeDesc, bool)> = Vec::new();
        let mut root_node_name = None;
        for (node_name, node_desc) in &graph_desc.nodes {
            match node_desc {
                NodeDesc::Root => {
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
            commands
                .entity(entity)
                .try_remove::<NotYetExtacted<AnimationGraphSource>>();
            return;
        };

        // Make animation graph
        let mut graph = AnimationGraph::new();
        let mut clip_name_to_node_idx = HashMap::new();
        let mut clip_name_to_handle = HashMap::new();
        let mut node_indices = HashMap::new();
        node_indices.insert(root_node_name.clone(), graph.root);

        let o_scene_armature_bone_paths = q_children
            .iter_descendants(entity)
            .find_map(|child| q_scene_root.get(child).map(|(_, sabp)| sabp.clone()).ok());

        // - Make mask groups
        let available_mask_group_idx = {
            let mut bone_name_to_target_ids = HashMap::new();
            if let Some(sabp) = o_scene_armature_bone_paths {
                for abp in sabp.armature_bone_paths.iter() {
                    for (bone_name, bone_name_paths) in abp.bone_paths.iter() {
                        let a_name = Name(abp.armature.to_owned().into());
                        let mut names = vec![a_name];
                        names.extend(
                            bone_name_paths
                                .iter()
                                .map(|name| Name(name.clone().into()))
                                .to_owned(),
                        );
                        let target_id = AnimationTargetId::from_names(names.iter());
                        bone_name_to_target_ids.insert(bone_name.to_owned(), target_id);
                    }
                }
            }
            let mut available_mask_group_idx = 0u64;
            for (idx, mask_group_desc) in graph_desc.mask_groups.0.iter().enumerate() {
                let mask_group_idx = graph_desc::MaskGroupIdx::new(idx as u32);
                let mask_target_ids = mask_group_desc
                    .targets
                    .iter()
                    .filter_map(|target| bone_name_to_target_ids.get(target))
                    .collect::<Vec<_>>();

                for mask_target_id in mask_target_ids {
                    debug!(
                        "add mask: {:?} to {:?}",
                        mask_target_id,
                        mask_group_idx.idx()
                    );
                    graph.add_target_to_mask_group(*mask_target_id, mask_group_idx.idx());
                }

                available_mask_group_idx |= mask_group_idx.bit();
            }
            available_mask_group_idx
        };

        let get_mask_bits = move |mask: &Vec<MaskGroupIdx>| {
            let mask_bits = mask.iter().fold(0u64, |acc, mask_idx| acc | mask_idx.bit());
            if mask_bits & !available_mask_group_idx != 0 {
                Err(format!(
                    "Mask groups {:?} are not defined in the graph: {:?}",
                    mask, graph_desc.mask_groups.0
                ))
            } else {
                Ok(mask_bits)
            }
        };
        // - Make nodes
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
                    NodeDesc::Clip(clip_node_desc) => {
                        let Some(h_clip) = gltf.named_animations.get(clip_node_desc.clip.as_str())
                        else {
                            error!("Clip {} not found in GLTF animations", clip_node_desc.clip);
                            continue;
                        };
                        let node_index = if clip_node_desc.mask.is_empty() {
                            graph.add_clip(h_clip.clone(), clip_node_desc.weight, parent_index)
                        } else {
                            let mask_bits = match get_mask_bits(&clip_node_desc.mask) {
                                Ok(bits) => bits,
                                Err(err) => {
                                    error!("ClipNode {}: {}", node_name, err);
                                    continue;
                                }
                            };

                            graph.add_clip_with_mask(
                                h_clip.clone(),
                                mask_bits,
                                clip_node_desc.weight,
                                parent_index,
                            )
                        };

                        clip_name_to_node_idx
                            .entry(clip_node_desc.clip.clone())
                            .or_insert_with(Vec::new)
                            .push(node_index);
                        clip_name_to_handle.insert(clip_node_desc.clip.clone(), h_clip.clone());

                        node_index
                    }
                    NodeDesc::Blend(blend_node_desc) => {
                        if blend_node_desc.mask.is_empty() {
                            graph.add_blend(blend_node_desc.weight, parent_index)
                        } else {
                            let mask_bits = match get_mask_bits(&blend_node_desc.mask) {
                                Ok(bits) => bits,
                                Err(err) => {
                                    error!("BlendNode {}: {}", node_name, err);
                                    continue;
                                }
                            };

                            graph.add_blend_with_mask(
                                mask_bits,
                                blend_node_desc.weight,
                                parent_index,
                            )
                        }
                    }
                    NodeDesc::AdditiveBlend(additive_blend_node_desc) => {
                        if additive_blend_node_desc.mask.is_empty() {
                            graph.add_additive_blend(additive_blend_node_desc.weight, parent_index)
                        } else {
                            let mask_bits = match get_mask_bits(&additive_blend_node_desc.mask) {
                                Ok(bits) => bits,
                                Err(err) => {
                                    error!("AdditiveBlendNode {}: {}", node_name, err);
                                    continue;
                                }
                            };

                            graph.add_additive_blend_with_mask(
                                mask_bits,
                                additive_blend_node_desc.weight,
                                parent_index,
                            )
                        }
                    }
                    NodeDesc::Root => {
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
        let graph_helper =
            AnimationGraphHelper::new(node_indices, clip_name_to_node_idx, clip_name_to_handle);

        // check if the AnimationClips are prepared to add events
        for h_clip in graph_helper.node_idx_to_clip_handle().values() {
            if !animatnion_clips.contains(h_clip) {
                // it needs to wait for the preparation to be continued.
                info!(
                    "AnimationClip {:?} is not yet prepared for {:?}",
                    h_clip, entity
                );
                continue;
            }
        }

        // WARN: All commands for a description and mutations must be issued in the below code because we may cancel to process the description for some reason(e.g. not yet loaded gltf/animation clip) and wait for the preparation to be continued.

        // for events
        for event_desc in &graph_desc.events {
            let node_idx = graph_helper
                .node_id_to_idx()
                .get(event_desc.clipnode())
                .unwrap_or_else(|| {
                    panic!(
                        "target node {} (in {}) not found in animation graph",
                        event_desc.clipnode(),
                        event_desc.event()
                    )
                });
            let h_clip = graph_helper
                .node_idx_to_clip_handle()
                .get(node_idx)
                .unwrap_or_else(|| {
                    panic!(
                        "clip handle for node {} (in {}) not found in animation graph",
                        event_desc.clipnode(),
                        event_desc.event()
                    )
                });
            let mut clip = animatnion_clips.get_mut(h_clip).unwrap_or_else(|| {
                panic!(
                    "clip for node {} (in {}) not found in animation graph",
                    event_desc.clipnode(),
                    event_desc.event()
                )
            });
            clip.add_event(
                event_desc.time(),
                InnerClipEvent::new(
                    event_desc.event().to_string(),
                    entity,
                    *node_idx,
                    event_desc.time(),
                ),
            );
            debug!(
                "added event {} at time {} for node {} (entity {:?})",
                event_desc.event(),
                event_desc.time(),
                event_desc.clipnode(),
                entity
            );
        }
        commands.add_observer(
            |trigger: On<InnerClipEvent>,
             mut commands: Commands,
             q_link: Query<&LinkToAnimationPlayer>,
             q_player: Query<(Entity, &AnimationPlayer)>| {


                 info!(
                     "received event {} for entity {:?} at time {} (target node: {:?})",
                     trigger.event_name(),
                     trigger.entity(),
                     trigger.time(),
                     trigger.target_node()
                 );
                 let Ok(link_to_player) = q_link.get(trigger.entity()) else {
                     warn!(
                         "entity {:?} does not have LinkToAnimationPlayer component",
                         trigger.entity()
                     );
                     return;
                 };
                 let Ok((_entity, player)) = q_player.get(link_to_player.player_entity()) else {
                     warn!(
                         "entity {:?} does not have AnimationPlayer component",
                         trigger.entity()
                     );
                     return;
                 };

                 let animation =  player.animation(trigger.target_node())
                     .unwrap_or_else(|| {
                         panic!(
                             "animation for node {:?} not found in AnimationPlayer for entity {:?}",
                             trigger.target_node(),
                             trigger.entity()
                         )
                     });

                 let last_seek_time = animation.last_seek_time();
                 let seek_time = animation.seek_time();
                 let repeat = animation.repeat_mode() != RepeatAnimation::Never;
                 let completions = animation.completions();

                 // patterns:
                 // - completions > 1: it means the animation has completed at least once, so we can trigger the event
                 // - completions == 1 && repeat: we should check [last_seek_time.or(0), duration] and [0, seek_time] to see if the event time is in either range
                 // - completions == 1 && !repeat: we should check [last_seek_time.or(0), duration] to see if the event time is in that range
                 // - completions == 0: we should check [last_seek_time.or(0), seek_time] to see if the event time is in that range

                 let should_trigger = if completions > 1 {
                     true
                 } else if completions == 1 {
                     if repeat {
                         (last_seek_time.is_none_or(|lst| lst < trigger.time())
                             && trigger.time() <= seek_time)
                             || (trigger.time() <= seek_time
                                 || last_seek_time.is_some_and(|lst| lst < trigger.time()))
                     } else {
                         last_seek_time.is_none_or(|lst| lst < trigger.time())
                             && trigger.time() <= seek_time
                     }
                 } else {
                     last_seek_time.is_none_or(|lst| lst < trigger.time())
                         && trigger.time() <= seek_time
                 };
                 if should_trigger {
                     debug!(
                         "triggering event {} for entity {:?} at time {} (last_seek_time: {:?}, seek_time: {}, completions: {}, repeat: {})",
                         trigger.event_name(),
                         trigger.entity(),
                         trigger.time(),
                         last_seek_time,
                         seek_time,
                         completions,
                         repeat
                     );
                     commands.entity(trigger.entity()).trigger(|e| {
                         ClipNodeEvent::new(e, trigger.event_name().to_string(), trigger.clone())
                     });
                 }
             },
        );

        // for commands
        let mut defined_commands = HashMap::new();
        for desc in &graph_desc.commands {
            use graph_desc::command_desc;
            let mut command_list = Vec::new();
            for command_desc in desc.list() {
                let node_idx = *graph_helper
                    .node_id_to_idx()
                    .get(command_desc.target_node())
                    .unwrap_or_else(|| {
                        panic!(
                            "target node {} (in {}) not found in animation graph",
                            command_desc.target_node(),
                            desc.name()
                        )
                    });

                let command: command::Command = match command_desc {
                    command_desc::CommandDesc::Play(play) => command::Command::Play(
                        command::Play::new(node_idx, play.repeat(), play.speed()),
                    ),
                    command_desc::CommandDesc::Stop(_stop) => {
                        command::Command::Stop(command::Stop::new(node_idx))
                    }
                    command_desc::CommandDesc::SetWeight(set_weight) => {
                        command::Command::SetWeight(command::SetWeight::new(
                            node_idx,
                            set_weight.weight(),
                        ))
                    }
                };
                command_list.push(command);
            }

            defined_commands.insert(desc.name().to_string(), command_list);
        }
        commands
            .entity(entity)
            .try_insert(command::DefinedAnimationGraphCommands::new(defined_commands));

        // for graph
        commands.entity(link_to_player.player_entity()).try_insert((
            graph_helper,
            AnimationGraphHandle(animation_graphs.add(graph)),
        ));

        // processed!
        commands
            .entity(entity)
            .try_remove::<NotYetExtacted<AnimationGraphSource>>();
    }
}
