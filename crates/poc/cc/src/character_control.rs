use std::f32::consts::PI;

use avian3d::{
    collision::collider::Collider,
    dynamics::rigid_body::{LockedAxes, RigidBody},
    prelude::*,
    spatial_query::{ShapeCaster, ShapeHits},
};
use bevy::prelude::*;

pub struct CharacterControlPlugin;

impl Plugin for CharacterControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<MovmentIndication>()
            .add_message::<AthleticNotification>()
            .add_systems(
                Update,
                (
                    keyboard_input,
                    update_grounded,
                    update_langing_and_push_off,
                    movement,
                    debug_print_grounded,
                    debug_print_notifications,
                )
                    .chain(),
            );
    }
}

pub fn spawn_sample_character(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
) {
    let cap_length = 0.7;
    let cap_radius = 0.15;
    let length = cap_length + cap_radius * 2.;
    let app_trans = Transform::from_xyz(0.0, (cap_radius*2. + cap_length) / 2., 0.0);
    let col_id = commands.spawn
        (
            // collider
            (
                Collider::capsule(cap_radius, cap_length),
                app_trans,
            )
        )
        .id();
    let id = commands
        .spawn((
            AthleticBundle::new(col_id, Collider::capsule(cap_radius, cap_length), app_trans, 0.2),
            Transform::from_xyz(0.0, length * 3., 0.0), // Spawn the character above the ground
            InheritedVisibility::VISIBLE,
        ))
        .add_child(col_id)
        .with_child(
            // appearance
            (
                Mesh3d(meshes.add(Capsule3d::new(cap_radius, cap_length))),
                MeshMaterial3d(standard_materials.add(StandardMaterial {
                    base_color: Color::srgb(0.8, 0.7, 0.6),
                    ..default()
                })),
                app_trans,
            ),
        ).id();

    // res
    commands.insert_resource(PlayerCharacter {
        control: id,
    });
}


#[derive(Component, Debug)]
pub struct AthleticController;

#[derive(Message, Debug)]
pub struct MovmentIndication {
    pub athrethic_base: Entity,
    pub action: MovementAction,
}

#[derive(Debug, Clone)]
pub enum MovementAction {
    Jump,
}

#[derive(Message, Debug)]
pub struct AthleticNotification {
    pub athrethic_base: Entity,
    pub info: AthleticNotificationInfo,
}

#[derive(Debug, Clone)]
pub enum AthleticNotificationInfo {
    StartLanding,
    StartPushOff,
    StartStanding,
    StartInAir { is_jumping: bool },
}

/// Component to indicate the grounded state of the character
#[derive(Component, Debug)]
pub enum Grounded {
    Landing { elapsed: f32 },
    Standing,
    PushOff { elapsed: f32 },
}

#[derive(Component, Debug)]
pub struct JumpImpulse(f32);


#[derive(Bundle)]
pub struct AthleticBundle {
    rigitbody: RigidBody,
    base: AthleticController,
    locked_axes: LockedAxes,
    grounding_caster: ShapeCaster,
    jump_impulse: JumpImpulse,
}

impl AthleticBundle {
    pub fn new(collider_entity: Entity, collider: Collider, collider_trans: Transform, jump_impulse: f32) -> Self {
        let mut caster_shape = collider.clone();
        caster_shape.set_scale(Vec3::ONE * 0.99, 10);
        let grounding_caster =
            ShapeCaster::new(caster_shape, collider_trans.translation, collider_trans.rotation, Dir3::NEG_Y)
            .with_max_distance(0.05)
            .with_query_filter(SpatialQueryFilter::from_excluded_entities([collider_entity]))
            ;
        Self {
            rigitbody: RigidBody::Dynamic,
            base: AthleticController,
            locked_axes: LockedAxes::ROTATION_LOCKED,
            grounding_caster,
            jump_impulse: JumpImpulse(jump_impulse),
        }
    }
}

#[derive(Resource, Debug)]
pub struct PlayerCharacter {
    control: Entity,
}

fn keyboard_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    player_character: Res<PlayerCharacter>,
    mut movement_writer: MessageWriter<MovmentIndication>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        movement_writer.write(MovmentIndication {
            athrethic_base: player_character.control,
            action: MovementAction::Jump,
        });
    }
}

fn update_grounded(
    time: Res<Time>,
    mut commands: Commands,
    q_athletic_base: Query<(
        Entity,
        &AthleticController,
        &ShapeHits,
        &GlobalTransform,
        &LinearVelocity,
        Option<&mut Grounded>,
    )>,
    mut notification_writer: MessageWriter<AthleticNotification>,
    mut l_last: Local<bool>,
) {
    for (entity, base, hits, global_trans, velocity, o_grounded) in &q_athletic_base {
        let has_upper_velocity = velocity.0.y > 0.0;
        let is_grounded = !has_upper_velocity && hits.iter().any(|hit| {
            const MAX_ANGLE: f32 = PI * 0.45;
            Vec3::Y.angle_between(-hit.normal2).abs() <= MAX_ANGLE
        });

        if !hits.is_empty() {
            //info!("update_grounded: entity = {:?}, is_grounded = {}, o_grounded = {:?}, hits={:?}", entity, is_grounded, o_grounded, hits);
        }

        if *l_last != is_grounded {
            info!(
                "update_grounded: entity = {:?}, is_grounded = {}, o_grounded = {:?}, hits={:?}",
                entity, is_grounded, o_grounded, hits
            );
        }
        *l_last = is_grounded;

        match (o_grounded, is_grounded) {
            (Some(Grounded::Landing { elapsed }), true) => {
                commands.entity(entity).try_insert(Grounded::Landing {
                    elapsed: *elapsed + time.delta_secs(),
                });
            }
            (Some(Grounded::Standing), false) => {
                // No change
            }
            (Some(Grounded::PushOff { elapsed }), true) => {
                commands.entity(entity).try_insert(Grounded::PushOff {
                    elapsed: *elapsed + time.delta_secs(),
                });
            }
            (None, true) => {
                commands
                    .entity(entity)
                    .try_insert(Grounded::Landing { elapsed: 0.0 });
                notification_writer.write(AthleticNotification {
                    athrethic_base: entity,
                    info: AthleticNotificationInfo::StartLanding,
                });
            }
            (None, false) => {

                commands.entity(entity).try_remove::<Grounded>();

                if o_grounded.is_some() {
                    notification_writer.write(AthleticNotification {
                        athrethic_base: entity,
                        info: AthleticNotificationInfo::StartInAir { is_jumping: false },
                    });
                }
            }
            _ => {}
        }
    }
}

// TODO: It should use the time of the landing motion & push off motion to determine when to change the state, instead of a hardcoded value
// I'm planing to use the events from Gltf animation for this
fn update_langing_and_push_off(
    mut commands: Commands,
    mut q_athletic_base: Query<(Entity, &AthleticController, &Grounded, &JumpImpulse, Forces)>,
    mut notification_writer: MessageWriter<AthleticNotification>,
) {
    //info!("update_langing_and_push_off: q_athletic_base.len() = {}", q_athletic_base.iter().len());
    const LANDING_DURATION: f32 = 0.1;
    const PUSH_OFF_DURATION: f32 = 0.1;
    for (entity, _base, grounded, jump_impulse, mut force) in &mut q_athletic_base {
        match grounded {
            Grounded::Landing { elapsed } if *elapsed > LANDING_DURATION => {
                commands.entity(entity).try_insert(Grounded::Standing);
                notification_writer.write(AthleticNotification {
                    athrethic_base: entity,
                    info: AthleticNotificationInfo::StartStanding,
                });
            }
            Grounded::PushOff { elapsed } if *elapsed > PUSH_OFF_DURATION => {
                commands.entity(entity).try_remove::<Grounded>();

                // Jump
                force.apply_linear_impulse(Vec3::Y * jump_impulse.0);

                notification_writer.write(AthleticNotification {
                    athrethic_base: entity,
                    info: AthleticNotificationInfo::StartInAir { is_jumping: true },
                });
            }
            _ => {}
        }
    }
}

fn movement(
    mut commands: Commands,
    mut q_athletic_base: Query<(Entity, &AthleticController, Option<&Grounded>)>,
    mut movement_reader: MessageReader<MovmentIndication>,
    mut notification_writer: MessageWriter<AthleticNotification>,
) {
    for movement in movement_reader.read() {
        let Ok((entity, _base, o_grounded)) = q_athletic_base.get_mut(movement.athrethic_base)
        else {
            continue;
        };
        if let (MovementAction::Jump, Some(Grounded::Standing)) =
            (movement.action.clone(), o_grounded)
        {
            commands
                .entity(entity)
                .try_insert(Grounded::PushOff { elapsed: 0.0 });
            notification_writer.write(AthleticNotification {
                athrethic_base: entity,
                info: AthleticNotificationInfo::StartPushOff,
            });
        }
    }
}

fn debug_print_grounded(q_athletic_base: Query<(Entity, &AthleticController, Option<&Grounded>, &GlobalTransform)>) {
    for (entity, _base, o_grounded, g_trans) in &q_athletic_base {
        info!(
            "AthleticController: {:?} - {:?} - {:?}",
            entity, o_grounded, g_trans.translation());
    }
}

fn debug_print_notifications(mut notification_reader: MessageReader<AthleticNotification>) {
    for notification in notification_reader.read() {
        info!(
            "AthleticNotification: {:?} - {:?}",
            notification.athrethic_base, notification.info
        );
    }
}
