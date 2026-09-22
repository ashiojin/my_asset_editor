use crate::character_control::*;
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
                // wait_load_gltf.run_if(in_state(SampleState::GltfLoading)),
                // wait_essentials_loaded.run_if(in_state(SampleState::EssentialsLoading)),
                // spawn_sample_character.run_if(in_state(SampleState::EssentialsLoaded)),
                // init_animation.run_if(in_state(SampleState::WorldAssetSpawned)),
                // kick_idle_animation.run_if(in_state(SampleState::AnimationInitialized)),
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

#[derive(Resource, Debug)]
struct SampleStore {
}

fn start_load_gltf(mut commands: Commands, asset_server: Res<AssetServer>) {
    const GLTF_PATH: &str = "models/inv_legs.glb";
    let h_gltf = asset_server.load::<Gltf>(GLTF_PATH);
    commands.insert_resource(GltfStore { h_gltf });
    commands.set_state(SampleState::WaitLoading);
}

fn spawn_sample_character(mut commands: Commands, store: Res<GltfStore>) {
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
    let id = commands
        .spawn((
            AthleticBundle::new(
                col_id,
                Collider::capsule(cap_radius, cap_length),
                col_trans,
                0.2,
            ),
            Transform::from_xyz(0.0, length * 3., 0.0), // Spawn the character above the ground
            InheritedVisibility::VISIBLE,
        ))
        .add_child(col_id)
        .with_child(
            // appearance
            (
                // Mesh3d(meshes.add(Capsule3d::new(cap_radius, cap_length))),
                // MeshMaterial3d(standard_materials.add(StandardMaterial {
                //     base_color: Color::srgb(0.8, 0.7, 0.6),
                //     ..default()
                // })),
                //col_trans,
                ashiojin_extensions::AshiojinGltfScene::new(store.h_gltf.clone(), ashiojin_extensions::GltfSceneLabel::Idx(0)),
            ),
        )
        .id();

    // res
    commands.insert_resource(PlayerCharacter { control: id });

    commands.set_state(SampleState::Idle);
}
#[derive(Resource, Debug)]
struct AnimationGraphNodes {
    idle: AnimationNodeIndex,
    inair: AnimationNodeIndex,
    landing: AnimationNodeIndex,
    takeoff: AnimationNodeIndex,
}

// fn init_animation(
//     mut commands: Commands,
//     q_anim_player: Query<(Entity, &AnimationPlayer), With<AnimationPlayer>>,
//     // We spawn only one WorldAssetRoot,
//     // so the animation player is the player of the character
//     mut graphs: ResMut<Assets<AnimationGraph>>,
//     sample_store: Res<SampleStore>,
// ) {
//     if let Ok((entity, _anim_player)) = q_anim_player.single() {
//         let mut graph = AnimationGraph::new();
//         let idle = graph.add_clip(sample_store.h_anime_idle.clone(), 1.0, graph.root);
//         let inair =  graph.add_clip(sample_store.h_anime_inair.clone(), 1.0, graph.root);
//         let landing = graph.add_clip(sample_store.h_anime_landing.clone(), 1.0, graph.root);
//         let takeoff = graph.add_clip(sample_store.h_anime_takeoff.clone(), 1.0, graph.root);
//         commands
//             .entity(entity)
//             .insert(AnimationGraphHandle(graphs.add(graph)));
//
//         commands.insert_resource(AnimationGraphNodes {
//             idle,
//             inair,
//             landing,
//             takeoff,
//         });
//
//         commands.set_state(SampleState::AnimationInitialized);
//     }
// }
//
//
// fn kick_idle_animation(
//     mut commands: Commands,
//     mut q_anim_player: Query<(&mut AnimationPlayer)>,
//     nodes: Res<AnimationGraphNodes>,
// ) {
//     if let Ok((mut anim_player)) = q_anim_player.single_mut() {
//         anim_player.play(nodes.idle).repeat();
//         commands.set_state(SampleState::Idle);
//     }
// }
