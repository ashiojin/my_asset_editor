use avian3d::prelude::*;
use bevy::{color::palettes::css, input::keyboard::Key, math::VectorSpace, prelude::*};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            ashiojin_extensions::AshiojinGltfExtensionsHandlerPlugin,
            PhysicsPlugins::default(),
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
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 4.0, 7.0).looking_at(Vec3::ZERO, Dir3::Y),
    ));

    commands.spawn((DirectionalLight {
        color: css::WHITE.into(),
        illuminance: 500.,
        shadow_maps_enabled: true,
        ..default()
    },));

    let ground = Vec3::new(10.0, 0.1, 10.0);
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
    let kabe_z = Vec3::new(10.0, 2.0, 0.1);
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::from_size(kabe_z))),
        MeshMaterial3d(standard_materials.add(StandardMaterial {
            base_color: css::LIGHT_GRAY.into(),
            ..default()
        })),
        Transform::from_xyz(0.0, kabe_z.y / 2., -10.0 / 2.),
        RigidBody::Static,
        Collider::cuboid(kabe_z.x, kabe_z.y, kabe_z.z),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::from_size(kabe_z))),
        MeshMaterial3d(standard_materials.add(StandardMaterial {
            base_color: css::LIGHT_GRAY.into(),
            ..default()
        })),
        Transform::from_xyz(0.0, kabe_z.y / 2., 10.0 / 2.),
        RigidBody::Static,
        Collider::cuboid(kabe_z.x, kabe_z.y, kabe_z.z),
    ));
    let kabe_x = Vec3::new(0.1, 2.0, 10.0);
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::from_size(kabe_x))),
        MeshMaterial3d(standard_materials.add(StandardMaterial {
            base_color: css::LIGHT_GRAY.into(),
            ..default()
        })),
        Transform::from_xyz(-10.0 / 2., kabe_x.y / 2., 0.0),
        RigidBody::Static,
        Collider::cuboid(kabe_x.x, kabe_x.y, kabe_x.z),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::from_size(kabe_x))),
        MeshMaterial3d(standard_materials.add(StandardMaterial {
            base_color: css::LIGHT_GRAY.into(),
            ..default()
        })),
        Transform::from_xyz(10.0 / 2., kabe_x.y / 2., 0.0),
        RigidBody::Static,
        Collider::cuboid(kabe_x.x, kabe_x.y, kabe_x.z),
    ));

    let sample_cube = Vec3::new(1.0, 1.0, 1.0);
    commands.spawn((
        Sample,
        Mesh3d(meshes.add(Cuboid::from_size(sample_cube))),
        MeshMaterial3d(standard_materials.add(StandardMaterial {
            base_color: css::RED.into(),
            ..default()
        })),
        Transform::from_xyz(0.0, 10.0, 3.0).with_rotation(Quat::from_rotation_x(0.25)),
        RigidBody::Dynamic,
        Collider::cuboid(sample_cube.x, sample_cube.y, sample_cube.z),
    ));
}

fn jump_up_samples(
    mut q_sample: Query<(Entity, Forces), With<Sample>>,

    key_input: Res<ButtonInput<Key>>,

    time: Res<Time>,
) {
    if key_input.just_pressed(Key::Space) {
        for (_sample_entity, mut forces) in &mut q_sample {


            let v = Vec3::Y * 5.0;
            forces.apply_linear_impulse(v);
        }
    }

    if key_input.just_pressed(Key::Character("b".into())) {
        for (_sample_entity, mut forces) in &mut q_sample {
            let s = time.elapsed_secs().sin();
            let c = time.elapsed_secs().cos();
            let torque = Vec3::new(s, 0.0, c) * 100.;
            forces.apply_torque(torque);
        }
    }
}
