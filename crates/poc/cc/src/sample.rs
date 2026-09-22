use crate::character_control::*;
use ashiojin_extensions::animation::{
    AnimationGraphHelper, AnimationGraphSource, LinkToAnimationPlayer,
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
                // wait_load_gltf.run_if(in_state(SampleState::GltfLoading)),
                // wait_essentials_loaded.run_if(in_state(SampleState::EssentialsLoading)),
                // spawn_sample_character.run_if(in_state(SampleState::EssentialsLoaded)),
                // init_animation.run_if(in_state(SampleState::WorldAssetSpawned)),
                // kick_idle_animation.run_if(in_state(SampleState::AnimationInitialized)),
                spawn_sample_character.run_if(in_state(SampleState::WaitLoading)),
                play_idle_of_sample.run_if(in_state(SampleState::Idle)),
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
struct SampleStore {}

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
    let app_id = commands
        .spawn(
            // appearance
            (
                // Mesh3d(meshes.add(Capsule3d::new(cap_radius, cap_length))),
                // MeshMaterial3d(standard_materials.add(StandardMaterial {
                //     base_color: Color::srgb(0.8, 0.7, 0.6),
                //     ..default()
                // })),
                //col_trans,
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
        .add_child(app_id)
        .id();

    // res
    commands.insert_resource(PlayerCharacter {
        control: id,
        app_id,
    });

    commands.set_state(SampleState::Idle);
}

fn play_idle_of_sample(
    player_character: Res<PlayerCharacter>,
    q_link_to_player: Query<&LinkToAnimationPlayer>,
    mut q_player: Query<(
        &mut AnimationPlayer,
        &AnimationGraphHandle,
        &AnimationGraphHelper,
    )>,
    mut _animation_graph: ResMut<Assets<AnimationGraph>>, // For example to set weights of nodes, but not used in this function
    mut is_processed: Local<bool>,
) {
    if *is_processed {
        return;
    }
    if let Ok(link) = q_link_to_player.get(player_character.app_id)
        && let Ok((mut anim_player, _graph_handle, graph_helper)) =
            q_player.get_mut(link.player_entity())
    {
        let nid_idle = graph_helper
            .node_id_to_idx()
            .get("clip_0")
            .expect("Idle node not found");
        anim_player.play(*nid_idle).repeat();
        *is_processed = true;

        // e.g. set weights
        // let mut graph = _animation_graph.get_mut(_graph_handle).expect("AnimationGraph not found");
        // graph.get_mut(*nid_idle).expect("Idle node not found").weight = 0.25;
    } else {
        info!("AnimationPlayer not found for the sample character");
    }
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
