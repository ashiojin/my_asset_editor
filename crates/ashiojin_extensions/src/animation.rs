use bevy::{
    animation::AnimationTargetId, platform::collections::HashMap, prelude::*,
    world_serialization::WorldInstanceReady,
};

use crate::{
    SceneArmatureBonePaths, SourceGltfHandle, animation::graph_desc::AnimeGraphDesc,
    scene::NotYetExtacted,
};

pub mod graph_desc;

#[derive(Default, Debug)]
pub struct AnimationGraphPlugin {
    _debug_mode: bool, // TODO: add systems that check invariants when debug_mode is true
}

impl Plugin for AnimationGraphPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(scene_spawned);

        app.add_systems(Update, apply_anim_graph);
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
    /// Name of clip node -> its idx and clip name
    node_id_to_idx: HashMap<String, AnimationNodeIndex>,

    /// Clip name -> node idx
    clip_name_to_node_idx: HashMap<String, AnimationNodeIndex>,
}

impl AnimationGraphHelper {
    pub fn new(
        node_id_to_idx: HashMap<String, AnimationNodeIndex>,
        clip_name_to_node_idx: HashMap<String, AnimationNodeIndex>,
    ) -> Self {
        Self {
            node_id_to_idx,
            clip_name_to_node_idx,
        }
    }

    pub fn node_id_to_idx(&self) -> &HashMap<String, AnimationNodeIndex> {
        &self.node_id_to_idx
    }

    pub fn clip_name_to_node_idx(&self) -> &HashMap<String, AnimationNodeIndex> {
        &self.clip_name_to_node_idx
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
#[require(NotYetExtacted<AnimationGraphEx>)]
pub struct AnimationGraphEx(AnimeGraphDesc);

impl AnimationGraphEx {
    pub fn new(desc: AnimeGraphDesc) -> Self {
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
            &AnimationGraphEx,
            &LinkToAnimationPlayer,
            &SourceGltfHandle,
        ),
        With<NotYetExtacted<AnimationGraphEx>>,
    >,
    q_scene_root: Query<(Entity, &SceneArmatureBonePaths)>,
    q_children: Query<&Children>,
    mut animation_graphs: ResMut<Assets<AnimationGraph>>,
) {
    use graph_desc::*;

    for (entity, AnimationGraphEx(graph_desc), link_to_player, SourceGltfHandle(gltf_handle)) in
        &q_target
    {
        info!("{:?}", entity);
        let Some(gltf) = gltf.get(gltf_handle) else {
            error!("GLTF not loaded yet, cannot apply animation graph");
            commands
                .entity(entity)
                .try_remove::<NotYetExtacted<AnimationGraphEx>>();
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
                .try_remove::<NotYetExtacted<AnimationGraphEx>>();
            return;
        };

        // Make animation graph
        let mut graph = AnimationGraph::new();
        let mut clip_name_to_node_idx = HashMap::new();
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

                        clip_name_to_node_idx.insert(clip_node_desc.clip.clone(), node_index);
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

        commands.entity(link_to_player.player_entity()).try_insert((
            AnimationGraphHelper::new(node_indices, clip_name_to_node_idx),
            AnimationGraphHandle(animation_graphs.add(graph)),
        ));
        commands
            .entity(entity)
            .try_remove::<NotYetExtacted<AnimationGraphEx>>();

    }
}
