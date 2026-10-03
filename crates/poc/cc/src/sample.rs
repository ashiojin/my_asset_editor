use crate::character_control::*;
use ashiojin_extensions::animation::{
    AnimationGraphHelper, AnimationGraphSource, LinkToAnimationPlayer, graph_desc::event_desc::ClipNodeEventDefinitionDesc,
};
use avian3d::prelude::*;
use bevy::prelude::*;

pub struct SamplePlugin;

impl Plugin for SamplePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<SampleState>();
        app.add_systems(Startup, start_load_gltf);
        app.add_systems(
            Update,
            (
                spawn_sample_character.run_if(in_state(SampleState::WaitLoading)),
                //initailize_around_animation_clips.run_if(in_state(SampleState::Idle)),
                initailize_animation_events.run_if(in_state(SampleState::Idle)),
                // debug_animation_playing_state.run_if(in_state(SampleState::Idle)),
            ),
        );
    }
}

#[derive(States, Debug, PartialEq, Eq, Clone, Copy, Hash, Default)]
enum SampleState {
    #[default]
    NotLoaded,
    WaitLoading,
    Idle,
}

#[derive(Resource, Debug)]
struct GltfStore {
    pub h_gltf: Handle<Gltf>,
}

fn start_load_gltf(mut commands: Commands, asset_server: Res<AssetServer>) {
    const GLTF_PATH: &str = "models/inv_legs.glb";
    let h_gltf = asset_server.load::<Gltf>(GLTF_PATH);
    commands.insert_resource(GltfStore { h_gltf });
    commands.set_state(SampleState::WaitLoading);
}

fn get_sample_animation_graph() -> ashiojin_extensions::animation::graph_desc::AnimationGraphDesc {
    let json = include_str!("../assets/models/inv_legs.clips_jump.ag.json");
    let graph_desc: ashiojin_extensions::animation::graph_desc::AnimationGraphDesc =
        serde_json::from_str(json).unwrap();
    graph_desc
}

#[derive(Component, Debug)]
struct LinkToController {
    controller_entity: Entity,
}

impl LinkToController {
    pub fn new(controller_entity: Entity) -> Self {
        Self { controller_entity }
    }
    pub fn controller_entity(&self) -> Entity {
        self.controller_entity
    }
}

fn spawn_sample_character(mut commands: Commands, store: Res<GltfStore>, gltf: Res<Assets<Gltf>>) {
    if !gltf.contains(&store.h_gltf) {
        info!("Gltf not loaded yet, waiting...");
        return;
    }
    let cap_length = 0.7;
    let cap_radius = 0.15;
    let length = cap_length + cap_radius * 2.;
    let col_trans = Transform::from_xyz(0.0, (cap_radius * 2. + cap_length) / 2., 0.0);
    let col_id = commands
        .spawn(
            // collider
            (Collider::capsule(cap_radius, cap_length), col_trans),
        )
        .id();
    let app_id = commands
        .spawn(
            // appearance
            (
                ashiojin_extensions::AshiojinGltfScene::new(
                    store.h_gltf.clone(),
                    ashiojin_extensions::GltfSceneLabel::Idx(0),
                ),
                AnimationGraphSource::new(get_sample_animation_graph()),
            ),
        )
        .id();
    #[derive(Debug)]
    enum AnimeType {
        Idle,
        InAir,
        Landing,
        Crunting,
    }
    let play_animation = move |anim_type: AnimeType,
                               q_link_to_player: Query<&LinkToAnimationPlayer>,
                               mut q_player: Query<(
        &mut AnimationPlayer,
        &AnimationGraphHandle,
        &AnimationGraphHelper,
    )>| {
        info!("Received request to play animation: {:?}", anim_type);
        // play landing animation
        if let Ok(link) = q_link_to_player.get(app_id)
            && let Ok((mut anim_player, _graph_handle, graph_helper)) =
                q_player.get_mut(link.player_entity())
        {
            let nid_landing = graph_helper
                .node_id_to_idx()
                .get("clip_3")
                .expect("Landing node not found");
            let nid_inair = graph_helper
                .node_id_to_idx()
                .get("clip_2")
                .expect("InAir node not found");
            let nid_crunting = graph_helper
                .node_id_to_idx()
                .get("clip_1")
                .expect("Crunting node not found");
            let nid_idle = graph_helper
                .node_id_to_idx()
                .get("clip_0")
                .expect("Idle node not found");
            match anim_type {
                AnimeType::Idle => {
                    info!("Playing Idle animation");
                    anim_player.play(*nid_idle).repeat();
                    for nid in [*nid_inair, *nid_crunting, *nid_landing] {
                        anim_player.stop(nid);
                    }
                }
                AnimeType::InAir => {
                    info!("Playing InAir animation");
                    anim_player.play(*nid_inair).repeat();
                    for nid in [*nid_idle, *nid_crunting, *nid_landing] {
                        anim_player.stop(nid);
                    }
                }
                AnimeType::Landing => {
                    info!("Playing Landing animation");
                    let a = anim_player.start(*nid_landing);
                    info!("Landing animation started: {:?}", a);
                    for nid in [*nid_inair, *nid_crunting, *nid_idle] {
                        anim_player.stop(nid);
                    }
                }
                AnimeType::Crunting => {
                    info!("Playing Crunting animation");
                    anim_player.start(*nid_crunting);
                    for nid in [*nid_inair, *nid_landing, *nid_idle] {
                        anim_player.stop(nid);
                    }
                }
            }
        } else {
            info!("AnimationPlayer not found for the sample character");
        }
    };

    let id = commands
        .spawn((
            PlayerController,
            AthleticBundle::new(
                col_id,
                Collider::capsule(cap_radius, cap_length),
                col_trans,
                0.4,
            ),
            Transform::from_xyz(0.0, length * 8., 0.0), // Spawn the character above the ground
            InheritedVisibility::VISIBLE,
        ))
        .add_child(col_id)
        .add_child(app_id)
        .observe(
            move |_ev: On<StartLanding>,
                  q_link_to_player: Query<&LinkToAnimationPlayer>,
                  q_player: Query<(
                &mut AnimationPlayer,
                &AnimationGraphHandle,
                &AnimationGraphHelper,
            )>| {
                play_animation(AnimeType::Landing, q_link_to_player, q_player);
            },
        )
        .observe(
            move |_ev: On<StartTakeoff>,
                  q_link_to_player: Query<&LinkToAnimationPlayer>,
                  q_player: Query<(
                &mut AnimationPlayer,
                &AnimationGraphHandle,
                &AnimationGraphHelper,
            )>| {
                play_animation(AnimeType::Crunting, q_link_to_player, q_player);
            },
        )
        .observe(
            move |_ev: On<StartInAir>,
                  q_link_to_player: Query<&LinkToAnimationPlayer>,
                  q_player: Query<(
                &mut AnimationPlayer,
                &AnimationGraphHandle,
                &AnimationGraphHelper,
            )>| {
                play_animation(AnimeType::InAir, q_link_to_player, q_player);
            },
        )
        .observe(
            move |_ev: On<StartStanding>,
                  q_link_to_player: Query<&LinkToAnimationPlayer>,
                  q_player: Query<(
                &mut AnimationPlayer,
                &AnimationGraphHandle,
                &AnimationGraphHelper,
            )>| {
                play_animation(AnimeType::Idle, q_link_to_player, q_player);
            },
        )
        .id();

    commands.entity(app_id).insert(LinkToController::new(id));

    commands.set_state(SampleState::Idle);
}

fn initailize_animation_events(
    mut commands: Commands,
    q_linked: Query<(&LinkToAnimationPlayer, &LinkToController)>,
    mut q_player: Query<
        (
            Entity,
            &mut AnimationPlayer,
            &AnimationGraphHandle,
            &AnimationGraphHelper,
        ),
        Added<AnimationGraphHelper>,
    >,
) {
    for (entity, mut _anim_player, _graph_handle, _graph_helper) in &mut q_player {
        let Some((_, link_to_controller)) = q_linked
            .iter()
            .find(|(l_player, _)| l_player.player_entity() == entity)
        else {
            continue;
        };

        info!(
            "Initializing animation clips for entity: {:?}, controller entity: {:?}",
            entity,
            link_to_controller.controller_entity()
        );

        // TODO: It should be included in the AnimationGraphDesc because the events are dependent
        // on the animation graph: the node names, the timing of the events, etc.
        let event_desc = ashiojin_extensions::animation::AnimationGraphEventsDesc::new(vec![
            ClipNodeEventDefinitionDesc::new("landing-end".into(), "clip_3".into(), 1.25),
            ClipNodeEventDefinitionDesc::new("jumping".into(), "clip_1".into(), 0.25),
            ClipNodeEventDefinitionDesc::new("idle-0.5".into(), "clip_0".into(), 0.5), // it will be ignored because we don't handle this event in the observer
        ]);

        commands.entity(entity)
            .try_insert(event_desc)
            .observe(|
                trigger: On<ashiojin_extensions::animation::ClipNodeEvent>,
                mut commands: Commands,
                q_linked: Query<'_, '_, (&LinkToAnimationPlayer, &LinkToController)>,
                | {
                    info!("Received animation event: {:?}", trigger);
                    let Some((_, link_to_controller)) = q_linked
                        .iter()
                        .find(|(l_player, _)| l_player.player_entity() == trigger.entity())
                    else {
                        error!(
                            "LinkToController not found for entity: {:?} when handling event: {}",
                            trigger.entity(),
                            trigger.event_name()
                        );
                        return;
                    };

                    match trigger.event_name() {
                        "landing-end" => {
                            info!("Landing animation finished for entity: {:?}", trigger.entity());
                            commands
                                .entity(link_to_controller.controller_entity())
                                .get_standing();
                        }
                        "jumping" => {
                            info!("Crunting animation at jumping for entity: {:?}", trigger.entity());
                            commands
                                .entity(link_to_controller.controller_entity())
                                .jump();
                        }
                        _ => {
                            warn!(
                                "Ignored unknown event name: {} for entity: {:?}",
                                trigger.event_name(),
                                trigger.entity()
                            );
                        }
                    }
            })
            ;
    }
}

