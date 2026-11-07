use std::f32::consts::{PI, TAU};

use crate::character_control::{
    capability::{jump::{JumpCapable, StartCrunting, request_jump}, rc_mov::RcMoveCapable}, *,
};
use ashiojin_extensions::animation::AnimationGraphSource;
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

#[derive(Component, Debug)]
struct SampleLanding;

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

                Transform::from_rotation(Quat::from_rotation_y(PI)), //
            ),
        )
        .id();

    let id = commands
        .spawn((
            PlayerController,
            CharacterControllerBundle::new(
                // TODO: separate `StandingCapable`
                col_id,
                Collider::capsule(cap_radius, cap_length),
                col_trans,
                0.4,
            ),
            Transform::from_xyz(0.0, length * 8., 0.0), // Spawn the character above the ground
            InheritedVisibility::VISIBLE,

            // TODO:
            RcMoveCapable {
                max_speed: 3.0,
                acceleration: 10.0,
                max_angular_speed: PI*1.5,
                angular_acceleration: PI * 3.0,
            },

            // AngularDamping(1.0),
            // LinearDamping(1.0),
        ))
        .add_child(col_id)
        .add_child(app_id)
        .observe(move |_ev: On<GetGrounded>, mut commands: Commands| {
            commands
                .entity(app_id)
                .queue(
                    ashiojin_extensions::animation::AnimationGraphCommandRequest(
                        "into-landing".to_string(),
                    ),
                )
                .try_insert(SampleLanding);
        })
        .observe(move |_ev: On<LostGrounded>, mut commands: Commands| {
            commands
                .entity(app_id)
                .queue(
                    ashiojin_extensions::animation::AnimationGraphCommandRequest(
                        "into-inair".to_string(),
                    ),
                )
                .try_remove::<JumpCapable>();
        })
        .observe(move |_ev: On<StartCrunting>, mut commands: Commands| {
            commands
                .entity(app_id)
                .queue(
                    ashiojin_extensions::animation::AnimationGraphCommandRequest(
                        "into-takeoff".to_string(),
                    ),
                )
                .try_remove::<JumpCapable>();
        })
        .id();

    commands.entity(app_id).insert(LinkToController::new(id));
    commands
        .entity(app_id)
        .observe(
            |trigger: On<ashiojin_extensions::animation::ClipNodeEvent>,
             mut commands: Commands,
             q_link_to_controller: Query<&LinkToController>| {
                info!("Received animation event: {:?}", trigger);
                let link = q_link_to_controller
                    .get(trigger.entity())
                    .expect("LinkToController not found for the entity");

                match trigger.event_name() {
                    "landing-end" => {
                        info!(
                            "Landing animation finished for entity: {:?}",
                            trigger.entity()
                        );

                        commands.entity(trigger.entity()).queue(
                            ashiojin_extensions::animation::AnimationGraphCommandRequest(
                                "into-idle".to_string(),
                            ),
                        );

                        commands
                            .entity(link.controller_entity())
                            .try_remove::<SampleLanding>()
                            .try_insert(JumpCapable::default());
                    }
                    "jumping" => {
                        info!(
                            "Crunting animation at jumping for entity: {:?}",
                            trigger.entity()
                        );
                        commands
                            .entity(link.controller_entity())
                            .queue(request_jump);
                    }
                    _ => {
                        warn!(
                            "Ignored unknown event name: {} for entity: {:?}(controller:{:?})",
                            trigger.event_name(),
                            trigger.entity(),
                            link.controller_entity()
                        );
                    }
                }
            },
        );

    commands.set_state(SampleState::Idle);
}
