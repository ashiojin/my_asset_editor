use crate::character_control::*;
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
                //initailize_around_animation_clips.run_if(in_state(SampleState::Idle)),
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
        .observe(move |_ev: On<StartLanding>, mut commands: Commands| {
            commands.entity(app_id).queue(
                ashiojin_extensions::animation::AnimationGraphCommandRequest(
                    "into-landing".to_string(),
                ),
            );
        })
        .observe(move |_ev: On<StartTakeoff>, mut commands: Commands| {
            commands.entity(app_id).queue(
                ashiojin_extensions::animation::AnimationGraphCommandRequest(
                    "into-takeoff".to_string(),
                ),
            );
        })
        .observe(move |_ev: On<StartStanding>, mut commands: Commands| {
            commands.entity(app_id).queue(
                ashiojin_extensions::animation::AnimationGraphCommandRequest(
                    "into-idle".to_string(),
                ),
            );
        })
        .observe(move |_ev: On<StartInAir>, mut commands: Commands| {
            commands.entity(app_id).queue(
                ashiojin_extensions::animation::AnimationGraphCommandRequest(
                    "into-inair".to_string(),
                ),
            );
        })
        .id();

    commands.entity(app_id).insert(LinkToController::new(id));
    commands
        .entity(app_id)
        //.try_insert(event_desc)
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
                        commands.entity(link.controller_entity()).get_standing();
                    }
                    "jumping" => {
                        info!(
                            "Crunting animation at jumping for entity: {:?}",
                            trigger.entity()
                        );
                        commands.entity(link.controller_entity()).jump();
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
