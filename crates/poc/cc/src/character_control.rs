use std::f32::consts::PI;

use avian3d::{
    collision::collider::Collider,
    dynamics::rigid_body::{LockedAxes, RigidBody},
    prelude::*,
    spatial_query::{ShapeCaster, ShapeHits},
};
use bevy::{input::keyboard::Key, prelude::*};

use intension::PlayerIntention;

use crate::{
    character_control::{
        capability::{jump::handle_jump_intentions, rc_mov::handle_rc_move_intentions},
    },
    unsorted::MainCamera,
};

pub mod capability;
pub mod intension;

pub struct CharacterControlPlugin;

impl Plugin for CharacterControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<PlayerIntention>().add_systems(
            Update,
            (
                keyboard_input.run_if(any_with_component::<PlayerController>),
                update_grounded,
                handle_jump_intentions,
                handle_rc_move_intentions,
            )
                .chain(),
        );
    }
}

#[derive(Component, Debug)]
pub struct CharacterController {}
impl CharacterController {
    fn new() -> Self {
        Self {}
    }
}

#[derive(Component, Debug, Default)]
pub struct GroundingState {
    is_grounded: bool,
    time: f32,
}

#[derive(EntityEvent, Debug)]
pub struct GetGrounded(Entity);

#[derive(EntityEvent, Debug)]
pub struct LostGrounded(Entity);

#[derive(Component, Debug)]
pub struct JumpImpulse(f32);

#[derive(Bundle)]
pub struct CharacterControllerBundle {
    rigitbody: RigidBody,
    base: CharacterController,
    locked_axes: LockedAxes,
    grounding_caster: ShapeCaster,
    jump_impulse: JumpImpulse,
    restitution: Restitution,
    grounding_state: GroundingState,
}

impl CharacterControllerBundle {
    pub fn new(
        collider_entity: Entity,
        collider: Collider,
        collider_trans: Transform,
        jump_impulse: f32,
    ) -> Self {
        let mut caster_shape = collider.clone();
        caster_shape.set_scale(Vec3::ONE * 0.99, 10);
        let grounding_caster = ShapeCaster::new(
            caster_shape,
            collider_trans.translation,
            collider_trans.rotation,
            Dir3::NEG_Y,
        )
        .with_max_distance(0.20)
        .with_query_filter(SpatialQueryFilter::from_excluded_entities([
            collider_entity,
        ]));
        Self {
            rigitbody: RigidBody::Dynamic,
            base: CharacterController::new(),
            locked_axes: LockedAxes::ROTATION_LOCKED.unlock_rotation_y(),
            grounding_caster,
            jump_impulse: JumpImpulse(jump_impulse),
            restitution: Restitution::new(0.0).with_combine_rule(CoefficientCombine::Min),
            grounding_state: GroundingState::default(),
        }
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct PlayerController;

fn keyboard_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut movement_writer: MessageWriter<PlayerIntention>,
    main_camera: Single<&GlobalTransform, With<MainCamera>>,
) {
    let mut intention = PlayerIntention {
        jump: false,
        heading: intension::Heading(Vec3::ZERO),
    };
    if keyboard_input.just_pressed(KeyCode::Space) {
        intention.jump = true;
    }

    let forward = main_camera.forward().with_y(0.).normalize();
    let right = main_camera.right().with_y(0.).normalize();
    let wasd: [(KeyCode, Vec3); 4] = [
        (KeyCode::KeyW, forward),
        (KeyCode::KeyS, -forward),
        (KeyCode::KeyA, -right),
        (KeyCode::KeyD, right),
    ];

    let mut heading_accum = Vec3::ZERO;
    for (key, dir) in wasd {
        if keyboard_input.pressed(key) {
            heading_accum += dir;
        }
    }
    intention.heading.0 = heading_accum.normalize_or_zero();
    //info!("heading: {:?}/{:?}, camera: {:?}-{:?}, key: {:?}", heading_accum, intention.heading.0, forward, right, wasd.iter().map(|(k, _)| keyboard_input.pressed(*k)).collect::<Vec<_>>());

    movement_writer.write(intention);
}
fn update_grounded(
    time: Res<Time>,
    mut commands: Commands,
    mut q_athletic_base: Query<
        (Entity, &ShapeHits, &LinearVelocity, &mut GroundingState),
        With<CharacterController>,
    >,
    mut l_last: Local<bool>,
) {
    for (entity, hits, velocity, mut grounding_state) in &mut q_athletic_base {
        let has_upper_velocity = velocity.0.y > 0.001;
        let is_grounded = !has_upper_velocity
            && hits.iter().any(|hit| {
                const MAX_ANGLE: f32 = PI * 0.45;
                Vec3::Y.angle_between(-hit.normal2).abs() <= MAX_ANGLE
            });

        if *l_last != is_grounded {
            info!(
                "update_grounded: entity = {:?}, velocity = {:?}, is_grounded = {}, grounding_state = {:?}, hits={:?}",
                entity, velocity, is_grounded, grounding_state, hits
            );
        }
        *l_last = is_grounded;

        if grounding_state.is_grounded != is_grounded {
            grounding_state.is_grounded = is_grounded;
            grounding_state.time = 0.0;
            if is_grounded {
                commands.entity(entity).trigger(GetGrounded);
            } else {
                commands.entity(entity).trigger(LostGrounded);
            }
        } else {
            grounding_state.time += time.delta_secs();
        }
    }
}
