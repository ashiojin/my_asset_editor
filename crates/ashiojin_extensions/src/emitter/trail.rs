use bevy::{platform::collections::HashMap, prelude::*};
use my_meshes::{SplineTrail, SplineTrailPoint, TrailInterpolationMode};
use std::collections::VecDeque;

use crate::{
    SandboxActionFxConfig, SandboxMeshFxConfigExtension, SourceGltfHandle,
    animation::{AnimationGraphHelper, LinkToAnimationPlayer},
    common::NotYetExtacted,
};

pub fn apply_sandbox_fx_meshes(
    mut commands: Commands,
    #[allow(clippy::type_complexity)] query: Query<
        (
            Entity,
            &SandboxMeshFxConfigExtension,
            &Name,
            Option<&Mesh3d>,
        ),
        With<SandboxMeshFxConfigExtension>,
    >,
    q_root: Query<(Entity, &LinkToAnimationPlayer), With<SourceGltfHandle>>,
    q_helper: Query<&AnimationGraphHelper>,
    q_scene_root_: Query<
        (Entity, &SandboxActionFxConfig),
        With<NotYetExtacted<SandboxActionFxConfig>>,
    >,
    q_parent: Query<&ChildOf>,
    q_children: Query<&Children>,

    mut waiting: Local<HashMap<Entity, usize>>,
) {
    // Add TrailEmitter
    for (scene_entity, fx_config) in q_scene_root_.iter() {
        // The insertion of LinkToAnimationPlayer is after scene extraction. So we wait it.
        let Some((_root_entity, link_to_player)) = q_parent
            .iter_ancestors(scene_entity)
            .find_map(|entity| q_root.get(entity).ok())
        else {
            warn!(
                "Scene root entity {:?} has SandboxActionFxConfig but no LinkToAnimationPlayer",
                scene_entity
            );
            continue;
        };
        // `AnimationGraphHelper` can be inserted after the scene spawned. So we wait it.
        let Ok(graph_helper) = q_helper.get(link_to_player.player_entity()) else {
            waiting
                .entry(scene_entity)
                .and_modify(|count| {
                    *count = count.saturating_add(1);
                })
                .or_insert(1);
            let cnt = waiting.get(&scene_entity).copied().unwrap_or(0);
            const CNT_PER_INFORMATION: usize = 180; // Log information every 180 frames (3 seconds at 60fps) to avoid spamming the log.
            if cnt % CNT_PER_INFORMATION == 0 {
                info!(
                    "Scene root entity {:?} has LinkToAnimationPlayer but no AnimationGraphHelper",
                    scene_entity
                );
            }
            continue;
        };
        waiting.remove(&scene_entity);

        // All infomation required to apply the fx meshes are ready.
        // It will be completed to apply the fx configs. If not, it should be a bug.
        commands
            .entity(scene_entity)
            .try_remove::<NotYetExtacted<SandboxActionFxConfig>>();

        // Search all descendants of the scene root entity to find meshes with SandboxMeshFxConfigExtension.
        q_children
            .iter_descendants(scene_entity)
            .for_each(|entity| {
                if let Ok((_, mesh_fx_config_extension, name, _mesh3d)) = query.get(entity) {
                    if !mesh_fx_config_extension.is_fx_mesh {
                        info!(
                            "Skipping entity {:?} (name: {:?}) because is_fx_mesh is false",
                            entity,
                            name.as_str()
                        );
                        return;
                    }
                    // TODO: Should use `fx_type` to determine which effect to apply. For now, we only have one effect, so we ignore it.
                    // The below code assumes that the `fx_type` is always "trail", and we will add a TrailEmitter to the entity.
                    let mut timings = vec![];
                    for (action_name, fx_configs) in fx_config.maps.iter() {
                        let Some(anim_node_idx_list) = graph_helper
                            .clip_name_to_node_idx()
                            .get(action_name.as_str())
                        else {
                            continue;
                        };
                        for anim_node_idx in anim_node_idx_list {
                            let fx_configs = fx_configs
                                .iter()
                                .filter(|fx| fx.target_name == name.as_str())
                                .map(|fx| {
                                    TrailEmitterTiming::new(
                                        *anim_node_idx,
                                        fx.start_sec,
                                        fx.end_sec,
                                    )
                                });

                            timings.extend(fx_configs);
                        }
                    }

                    info!(
                        "Adding TrailEmitter to entity {:?} (name: {:?}) with timings: {:?}",
                        entity,
                        name.as_str(),
                        timings
                    );

                    commands.entity(entity).try_insert((
                        TrailEmitter::new(0.2).extend_timings(timings), // FIXME: `0.2` is a placeholder for lifetime. Should be configurable.
                        Visibility::Hidden,
                    ));
                }
            });
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TrailEmitterTiming {
    pub node_idx: AnimationNodeIndex,
    pub start_time: f32,
    pub end_time: f32,
}
impl TrailEmitterTiming {
    pub fn new(node_idx: AnimationNodeIndex, start_time: f32, end_time: f32) -> Self {
        Self {
            node_idx,
            start_time,
            end_time,
        }
    }

    pub fn node_idx(&self) -> AnimationNodeIndex {
        self.node_idx
    }

    pub fn is_on_time(&self, time: f32) -> bool {
        time >= self.start_time && time <= self.end_time
    }
}

#[derive(Component, Debug, Clone)]
pub struct TrailEmitter {
    lifetime: f32,
    timing: Vec<TrailEmitterTiming>,
    mode: TrailInterpolationMode,
    subdivisions: u32,
}

#[allow(dead_code)]
impl TrailEmitter {
    pub fn new(lifetime: f32) -> Self {
        Self {
            lifetime,
            timing: vec![],
            mode: TrailInterpolationMode::LinearLastSegment,
            subdivisions: 8,
        }
    }

    pub fn add_timing(mut self, timing: TrailEmitterTiming) -> Self {
        self.timing.push(timing);
        self
    }

    pub fn extend_timings(mut self, timings: Vec<TrailEmitterTiming>) -> Self {
        self.timing.extend(timings);
        self
    }

    pub fn with_mode(mut self, mode: TrailInterpolationMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn with_subdivisions(mut self, subdivisions: u32) -> Self {
        self.subdivisions = subdivisions;
        self
    }

    pub fn lifetime(&self) -> f32 {
        self.lifetime
    }

    pub fn timings(&self) -> &Vec<TrailEmitterTiming> {
        &self.timing
    }

    pub fn timings_of(&self, node_idx: AnimationNodeIndex) -> Vec<&TrailEmitterTiming> {
        self.timing
            .iter()
            .filter(|t| t.node_idx() == node_idx)
            .collect()
    }

    pub fn mode(&self) -> TrailInterpolationMode {
        self.mode
    }

    pub fn subdivisions(&self) -> u32 {
        self.subdivisions
    }
}

#[derive(Component, Debug, Clone)]
pub struct TrailHistory {
    pub points: VecDeque<SplineTrailPoint>,
    pub trail_entity: Option<Entity>,
    pub mode: TrailInterpolationMode,
    pub subdivisions: u32,
    /// Whether the emitter was emitting on the previous frame. Used to detect
    /// the start of a new burst (idle -> active), which must cut the trail.
    pub was_active: bool,
    /// Set when a new burst begins so the next pushed point is flagged
    /// `break_before`, even if the exact transition frame pushed no point.
    pub pending_break: bool,
}

impl TrailHistory {
    pub fn new(mode: TrailInterpolationMode, subdivisions: u32) -> Self {
        Self {
            points: VecDeque::new(),
            trail_entity: None,
            mode,
            subdivisions,
            was_active: false,
            pending_break: false,
        }
    }
}

#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
pub fn spawn_trail_from_emitter(
    mut commands: Commands,
    mut q_trail_emitter: Query<(
        Entity,
        &TrailEmitter,
        &Mesh3d,
        &GlobalTransform,
        Option<&mut TrailHistory>,
    )>,
    mut assets_meshes: ResMut<Assets<Mesh>>,
    q_mesh_3d: Query<&Mesh3d>,
    q_animation_players: Query<(Entity, &AnimationPlayer)>,
    q_children: Query<&ChildOf>,
    q_link_to_player: Query<(Entity, &LinkToAnimationPlayer)>,
    trail_config: Option<Res<TrailConfig>>,
    time: Res<Time>,

    //---
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let current_time = time.elapsed_secs();

    for (entity, trail_emitter, mesh, global_transform, opt_history) in q_trail_emitter.iter_mut() {
        let Some(mesh_asset) = assets_meshes.get(&mesh.0) else {
            continue;
        };

        // Here we assume there are two vertices
        let Some(vertices) = mesh_asset
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .and_then(|attr| attr.as_float3())
        else {
            continue;
        };
        if vertices.is_empty() {
            continue;
        }

        // For now we use only 2 vertices, first and last one, to determine the trail positions.
        let vertices = [vertices[0], vertices[vertices.len() - 1]];
        // Get global positions by transforming local vertices using the global transform
        let global_positions: Vec<Vec3> = vertices
            .iter()
            .map(|v| global_transform.transform_point(Vec3::from(*v)))
            .collect();

        let current_root = global_positions[0];
        let current_tip = global_positions[1];

        let mut spawn_trail = false;

        // Find AutoPlay from ancestors of the Entity
        let opt_auto_play = q_children
            .iter_ancestors(entity)
            .find_map(|e| q_link_to_player.get(e).map(|(_, auto_play)| auto_play).ok());

        // Check if the animations which have
        if let Some(link_to_player) = opt_auto_play
            && let Ok((_entity, player)) = q_animation_players.get(link_to_player.player_entity())
        {
            for (playing_anim_nidx, playing_animation) in player.playing_animations() {
                let seek_time = playing_animation.seek_time();
                if trail_emitter
                    .timings_of(*playing_anim_nidx)
                    .iter()
                    .any(|timing| timing.is_on_time(seek_time))
                {
                    debug!(
                        "TrailEmitter for entity {:?} is active at elapsed time {:?}, spawning trail",
                        entity,
                        playing_animation.seek_time()
                    );
                    spawn_trail = true;
                } else {
                    debug!(
                        "TrailEmitter for entity {:?} has no timing for playing animation node {:?}, skipping trail spawn",
                        entity, playing_anim_nidx
                    );
                }
            }
        } else {
            info!(
                "AnimationPlayer for entity {:?} does not exist or is not playing, skipping trail spawn",
                entity
            );
        }

        // Handle the history queue
        let mut history = match opt_history {
            Some(h) => h,
            None => {
                let initial_mode = trail_config
                    .as_ref()
                    .map(|c| c.mode)
                    .unwrap_or(trail_emitter.mode());
                let mut h = TrailHistory::new(initial_mode, trail_emitter.subdivisions());
                if spawn_trail {
                    h.points.push_back(SplineTrailPoint {
                        root: current_root,
                        tip: current_tip,
                        time: current_time,
                        break_before: false,
                    });
                    h.was_active = true;
                }
                commands.entity(entity).insert(h);
                continue;
            }
        };

        // Detect the start of a new burst: the emitter is active now but was
        // idle last frame. The first point pushed for this burst is flagged so
        // the mesh builder cuts the ribbon instead of bridging the idle gap.
        if spawn_trail && !history.was_active && !history.points.is_empty() {
            history.pending_break = true;
        }

        // Push new point if spawning is active
        if spawn_trail {
            // Avoid inserting identical positions at the exact same timestamp to prevent zero-length divisions
            let should_push = if let Some(last) = history.points.back() {
                last.root.distance_squared(current_root) > 1e-6
                    || last.tip.distance_squared(current_tip) > 1e-6
                    || (current_time - last.time) > 0.05
            } else {
                true
            };

            if should_push {
                let break_before = history.pending_break;
                history.points.push_back(SplineTrailPoint {
                    root: current_root,
                    tip: current_tip,
                    time: current_time,
                    break_before,
                });
                history.pending_break = false;
            }
        }

        // Remember this frame's emission state for next-frame gap detection.
        history.was_active = spawn_trail;

        // Prune old points
        let cutoff_time = current_time - trail_emitter.lifetime();
        while history.points.len() > 1 && history.points[0].time < cutoff_time {
            history.points.pop_front();
        }

        // Rebuild or clean up trail entity.
        //
        // `build_mesh` yields `None` when the history holds no drawable ribbon --
        // e.g. a break has just isolated the single point left over from the
        // previous burst. Such a frame must be treated exactly like an empty
        // history: pushing a zero-vertex mesh into `Assets<Mesh>` makes bevy's
        // MeshAllocator free the old allocation, skip re-allocating, and still
        // attempt the vertex copy, logging
        // "Use-after-free: attempted to copy element data for an unallocated key".
        let new_mesh = if history.points.len() >= 2 {
            SplineTrail::new(
                history.points.iter().cloned().collect(),
                history.subdivisions,
            )
            .build_mesh(history.mode)
        } else {
            None
        };

        if let Some(new_mesh) = new_mesh {
            if let Some(trail_ent) = history.trail_entity {
                if let Ok(mesh_3d) = q_mesh_3d.get(trail_ent)
                    && let Some(mut mesh_asset) = assets_meshes.get_mut(&mesh_3d.0)
                {
                    *mesh_asset = new_mesh;
                }
            } else {
                let mesh_handle = assets_meshes.add(new_mesh);
                let spawned_ent = commands
                    .spawn((
                        Mesh3d(mesh_handle),
                        Transform::IDENTITY,
                        GlobalTransform::IDENTITY,
                        // TODO: Material should be configurable.
                        MeshMaterial3d(materials.add(StandardMaterial {
                            base_color: Color::srgba(0.0, 1.0, 1.0, 0.5),
                            unlit: true,
                            ..Default::default()
                        })),
                    ))
                    .id();
                history.trail_entity = Some(spawned_ent);
                info!(
                    "Spawned new trail entity {:?} for emitter {:?}",
                    spawned_ent, entity
                );
            }
        } else {
            // Nothing drawable: no segments left (or faded completely)
            if let Some(trail_ent) = history.trail_entity {
                commands.entity(trail_ent).try_despawn();
                history.trail_entity = None;
            }
        }
    }
}

#[derive(Resource, Debug, Clone, Copy)]
pub struct TrailConfig {
    pub mode: TrailInterpolationMode,
}

impl Default for TrailConfig {
    fn default() -> Self {
        Self {
            mode: TrailInterpolationMode::LinearLastSegment,
        }
    }
}
