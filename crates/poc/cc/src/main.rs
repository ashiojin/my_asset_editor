use avian3d::prelude::*;
use bevy::{color::palettes::css, input::keyboard::Key, prelude::*};

mod character_control;
mod sample;

fn main() {
    let asset_root_path = std::env::var("ASSETS_DIR").unwrap_or("assets".into());
    let default_plugin = DefaultPlugins
        .set(AssetPlugin {
            file_path: asset_root_path,
            //watch_for_changes_override: Some(true),
            ..Default::default()
        })
        .build();
    App::new()
        .add_plugins((
            default_plugin,
            ashiojin_extensions::AshiojinGltfExtensionsHandlerPlugin,
            character_control::CharacterControlPlugin,
            sample::SamplePlugin,
            PhysicsPlugins::default(),
            PhysicsDebugPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, jump_up_samples)
        .run();
}

#[derive(Component, Debug)]
struct Ground;

#[derive(Component, Debug)]
struct Sample;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
) {
    let scale = 3.0;
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, scale * 4.0, scale * 7.0).looking_at(Vec3::ZERO, Dir3::Y),
    ));

    commands.spawn((
        DirectionalLight {
            color: css::WHITE.into(),
            illuminance: 1000.,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.0, 999.0, 0.0).looking_to(Dir3::NEG_Y, Dir3::Z),
    ));

    let ground_side_length = scale * 10.0;
    let kabe_height = scale * 1.2;
    let ground = Vec3::new(ground_side_length, 0.1, ground_side_length);
    commands.spawn((
        Ground,
        Mesh3d(meshes.add(Cuboid::from_size(ground))),
        MeshMaterial3d(standard_materials.add(StandardMaterial {
            base_color: css::GRAY.into(),
            ..default()
        })),
        Transform::from_xyz(0.0, -ground.y / 2., 0.0),
        RigidBody::Static,
        Collider::cuboid(ground.x, ground.y, ground.z),
    ));
    let kabe_z = Vec3::new(ground_side_length, kabe_height, 0.1);
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::from_size(kabe_z))),
        MeshMaterial3d(standard_materials.add(StandardMaterial {
            base_color: css::LIGHT_GRAY.into(),
            ..default()
        })),
        Transform::from_xyz(0.0, kabe_z.y / 2., -ground_side_length / 2.),
        RigidBody::Static,
        Collider::cuboid(kabe_z.x, kabe_z.y, kabe_z.z),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::from_size(kabe_z))),
        MeshMaterial3d(standard_materials.add(StandardMaterial {
            base_color: css::LIGHT_GRAY.into(),
            ..default()
        })),
        Transform::from_xyz(0.0, kabe_z.y / 2., ground_side_length / 2.),
        RigidBody::Static,
        Collider::cuboid(kabe_z.x, kabe_z.y, kabe_z.z),
    ));
    let kabe_x = Vec3::new(0.1, kabe_height, ground_side_length);
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::from_size(kabe_x))),
        MeshMaterial3d(standard_materials.add(StandardMaterial {
            base_color: css::LIGHT_GRAY.into(),
            ..default()
        })),
        Transform::from_xyz(-ground_side_length / 2., kabe_x.y / 2., 0.0),
        RigidBody::Static,
        Collider::cuboid(kabe_x.x, kabe_x.y, kabe_x.z),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::from_size(kabe_x))),
        MeshMaterial3d(standard_materials.add(StandardMaterial {
            base_color: css::LIGHT_GRAY.into(),
            ..default()
        })),
        Transform::from_xyz(ground_side_length / 2., kabe_x.y / 2., 0.0),
        RigidBody::Static,
        Collider::cuboid(kabe_x.x, kabe_x.y, kabe_x.z),
    ));

    let sample_cube = Vec3::new(1.0, 0.5, 1.0);
    commands.spawn((
        Sample,
        Mesh3d(meshes.add(Cuboid::from_size(sample_cube))),
        MeshMaterial3d(standard_materials.add(StandardMaterial {
            base_color: css::RED.into(),
            ..default()
        })),
        Transform::from_xyz(2.0, ground_side_length, 3.0)
            .with_rotation(Quat::from_rotation_x(0.25)),
        RigidBody::Dynamic,
        Collider::cuboid(sample_cube.x, sample_cube.y, sample_cube.z),
    ));
}

fn jump_up_samples(
    mut q_sample: Query<(Entity, Forces), With<Sample>>,

    key_input: Res<ButtonInput<Key>>,
) {
    if key_input.just_pressed(Key::Character("1".into())) {
        for (_sample_entity, mut forces) in &mut q_sample {
            let v = Vec3::Y * 5.0;
            forces.apply_linear_impulse(v);
        }
    }
}
