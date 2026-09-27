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
        app.add_message::<PlayerIndication>().add_systems(
            Update,
            (
                keyboard_input.run_if(any_with_component::<PlayerController>),
                update_grounded,
                //update_langing_and_push_off,
                movement,
                debug_print_grounded,
                //debug_print_notifications,
            )
                .chain(),
        );
    }
}

#[derive(Component, Debug)]
pub struct AthleticController {}
impl AthleticController {
    fn new() -> Self {
        Self {}
    }
}

#[derive(Message, Debug)]
pub struct PlayerIndication {
    pub action: PlayerIndicationType,
}

#[derive(Debug, Clone)]
pub enum PlayerIndicationType {
    Jump,
}

/// apply a jump impulse to the entity.
/// It is an EntityCommands
fn ecmd_jump(mut entity: EntityWorldMut) {
    let Ok((jump_impulse, mut forces)) = entity.get_components_mut::<(&JumpImpulse, Forces)>()
    else {
        error!(
            "entity {:?} does not have JumpImpulse and Forces components",
            entity.id()
        );
        error!(" - JumpImpluse : {}", entity.contains::<JumpImpulse>());
        error!(
            " - Forces : {}",
            entity.get_components_mut::<Forces>().is_ok()
        );
        error!("Check if the entity has the AthleticBundle components");
        return;
    };

    forces.apply_linear_impulse(Vec3::Y * jump_impulse.0);
}

fn ecmd_get_standing(mut entity: EntityWorldMut) {
    let Ok(mut grounded) = entity.get_components_mut::<&mut Grounded>() else {
        error!(
            "entity {:?} does not have Grounded and MessageWriter<AthleticNotification> components",
            entity.id()
        );
        error!("Check if the entity has the AthleticBundle components");
        return;
    };

    *grounded = Grounded::Standing;
    entity.trigger(StartStanding);
}

pub trait AthleticEntityCommandsExt {
    fn jump(&mut self) -> &mut Self;
    fn get_standing(&mut self) -> &mut Self;
}

impl AthleticEntityCommandsExt for EntityCommands<'_> {
    fn jump(&mut self) -> &mut Self {
        self.queue(ecmd_jump);
        self
    }
    fn get_standing(&mut self) -> &mut Self {
        self.queue(ecmd_get_standing);
        self
    }
}

#[derive(EntityEvent, Debug)]
pub struct StartLanding(Entity);

#[derive(EntityEvent, Debug)]
pub struct StartTakeoff(Entity);

#[derive(EntityEvent, Debug)]
pub struct StartStanding(Entity);

#[derive(EntityEvent, Debug)]
pub struct StartInAir(Entity);

/// Component to indicate the grounded state of the character
#[derive(Component, Debug)]
pub enum Grounded {
    Landing { elapsed: f32 },
    Standing,
    Crunting { elapsed: f32 },
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
    restitution: Restitution,
}

impl AthleticBundle {
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
            base: AthleticController::new(),
            locked_axes: LockedAxes::ROTATION_LOCKED,
            grounding_caster,
            jump_impulse: JumpImpulse(jump_impulse),
            restitution: Restitution::new(0.0)
                .with_combine_rule(CoefficientCombine::Min)
                ,
        }
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct PlayerController;

fn keyboard_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut movement_writer: MessageWriter<PlayerIndication>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        movement_writer.write(PlayerIndication {
            action: PlayerIndicationType::Jump,
        });
    }
}

fn update_grounded(
    time: Res<Time>,
    mut commands: Commands,
    q_athletic_base: Query<
        (Entity, &ShapeHits, &LinearVelocity, Option<&mut Grounded>),
        With<AthleticController>,
    >,
    mut l_last: Local<bool>,
) {
    for (entity, hits, velocity, o_grounded) in &q_athletic_base {
        let has_upper_velocity = velocity.0.y > 0.001;
        let is_grounded = !has_upper_velocity
            && hits.iter().any(|hit| {
                const MAX_ANGLE: f32 = PI * 0.45;
                Vec3::Y.angle_between(-hit.normal2).abs() <= MAX_ANGLE
            });

        if *l_last != is_grounded {
            info!(
                "update_grounded: entity = {:?}, velocity = {:?}, is_grounded = {}, o_grounded = {:?}, hits={:?}",
                entity, velocity, is_grounded, o_grounded, hits
            );
        }
        *l_last = is_grounded;

        match (o_grounded, is_grounded) {
            (Some(Grounded::Landing { elapsed }), true) => {
                commands.entity(entity).try_insert(Grounded::Landing {
                    elapsed: *elapsed + time.delta_secs(),
                });
            }
            (Some(Grounded::Crunting { elapsed }), true) => {
                commands.entity(entity).try_insert(Grounded::Crunting {
                    elapsed: *elapsed + time.delta_secs(),
                });
            }
            (None, true) => {
                // just landed
                commands
                    .entity(entity)
                    .try_insert(Grounded::Landing { elapsed: 0.0 })
                    .trigger(StartLanding);
            }
            (None, false) => {
                // still in air, no change
            }
            (Some(_), false) => {
                // at this point, the character is in the air
                commands
                    .entity(entity)
                    .try_remove::<Grounded>()
                    .trigger(StartInAir);
            }
            (Some(Grounded::Standing), true) => {
                // do nothing, still standing
            }
        }
    }
}

// TODO: It should use the time of the landing motion & push off motion to determine when to change the state, instead of a hardcoded value
// I'm planing to use the events from Gltf animation for this
// fn update_langing_and_push_off(
//     mut commands: Commands,
//     mut q_athletic_base: Query<(Entity, &Grounded), With<AthleticController>>,
// ) {
//     //info!("update_langing_and_push_off: q_athletic_base.len() = {}", q_athletic_base.iter().len());
//     const LANDING_DURATION: f32 = 0.1;
//     const PUSH_OFF_DURATION: f32 = 0.1;
//     for (entity, grounded) in &mut q_athletic_base {
//         match grounded {
//             Grounded::Landing { elapsed } if *elapsed > LANDING_DURATION => {
//                 commands.entity(entity).get_standing();
//             }
//             Grounded::Crunting { elapsed } if *elapsed > PUSH_OFF_DURATION => {
//                 commands.entity(entity).jump();
//             }
//             _ => {}
//         }
//     }
// }

fn movement(
    mut commands: Commands,
    mut q_athletic_base: Query<(Entity, &AthleticController, Option<&Grounded>), With<PlayerController>>,
    mut player_indication_reader: MessageReader<PlayerIndication>,
) {
    for movement in player_indication_reader.read() {
        for (entity, _base, o_grounded) in &mut q_athletic_base {
            if let (PlayerIndicationType::Jump, Some(Grounded::Standing)) =
                (movement.action.clone(), o_grounded)
            {
                commands
                    .entity(entity)
                    .try_insert(Grounded::Crunting { elapsed: 0.0 })
                    .trigger(StartTakeoff)
                    ;
            }
        }
        // let Ok((entity, _base, o_grounded)) = q_athletic_base.get_mut(player_character.control)
        // else {
        //     continue;
        // };
        // if let (PlayerIndicationType::Jump, Some(Grounded::Standing)) =
        //     (movement.action.clone(), o_grounded)
        // {
        //     commands
        //         .entity(entity)
        //         .try_insert(Grounded::Crunting { elapsed: 0.0 })
        //         .trigger(StartTakeoff)
        //         ;
        // }
    }
}

fn debug_print_grounded(
    q_athletic_base: Query<(
        Entity,
        &AthleticController,
        Option<&Grounded>,
        &GlobalTransform,
    )>,
) {
    for (entity, _base, o_grounded, g_trans) in &q_athletic_base {
        debug!(
            "AthleticController: {:?} - {:?} - {:?}",
            entity,
            o_grounded,
            g_trans.translation()
        );
    }
}

// fn debug_print_notifications(mut notification_reader: MessageReader<AthleticNotification>) {
//     for notification in notification_reader.read() {
//         info!(
//             "AthleticNotification: {:?} - {:?}",
//             notification.athrethic_base, notification.info
//         );
//     }
// }
