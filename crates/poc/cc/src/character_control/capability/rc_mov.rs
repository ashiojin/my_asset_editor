use avian3d::dynamics::rigid_body::forces::{Forces, ReadRigidBodyForces, WriteRigidBodyForces};
use bevy::{platform::collections::HashSet, prelude::*};

use crate::character_control::{CharacterController, intension::PlayerIntention};

#[derive(Component, Debug, Clone, Default)]
pub struct RcMoveCapable {
    tag: &'static str,
    max_speed: f32,
    acceleration: f32,

    max_angular_speed: f32,
    angular_acceleration: f32,
}

impl RcMoveCapable {
    pub fn new(
        tag: &'static str,
        max_speed: f32,
        acceleration: f32,
        max_angular_speed: f32,
        angular_acceleration: f32,
    ) -> Self {
        Self {
            tag,
            max_speed,
            acceleration,
            max_angular_speed,
            angular_acceleration,
        }
    }
}

// TODO: Events
// Candidates
// - Start moving
// - Start to stop moving
// - End moving

#[derive(Debug, Clone, PartialEq, Copy)]
#[expect(clippy::enum_variant_names, reason = "Start is a good prefix for the event types")]
pub enum RcEventType {
    /// The entity has started standing still
    StartStanding,
    /// The entity has started moving
    StartMoving,
    /// The entity has started stopping (intended to stop but still moving)
    StartStopping,
}

#[derive(EntityEvent, Debug)]
pub struct UpdateRcMove {
    entity: Entity,
    pub event: Option<RcEventType>,
    pub linear_velocity: Vec3,
    pub linear_acceleration: Vec3,
    pub angular_velocity: Vec3,
    pub angular_acceleration: Vec3,
}

pub fn handle_rc_move_intentions(
    mut commands: Commands,
    mut q_rc_move: Query<
        (Entity, Forces, &GlobalTransform, &RcMoveCapable),
        With<CharacterController>,
    >,
    mut player_indication_reader: MessageReader<PlayerIntention>,
    time: Res<Time>,
    mut gizmos: Gizmos,
) {
    let mut remaining_entities = q_rc_move
        .iter()
        .map(|(entity, _, _, _)| entity)
        .collect::<HashSet<_>>();
    let mut events = Vec::<UpdateRcMove>::new();

    // FIXME: more consideration for angular velocity and acceleration
    for intent in player_indication_reader.read() {
        let mut event = UpdateRcMove {
            entity: intent.entity,
            event: None,
            linear_velocity: Vec3::ZERO,
            linear_acceleration: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            angular_acceleration: Vec3::ZERO,
        };

        remaining_entities.retain(|&e| e != intent.entity);
        let Ok((_entity, mut forces, transform, rc_move)) = q_rc_move.get_mut(intent.entity) else {
            warn!(
                "entity {:?} does not have RcMoveCapable component",
                intent.entity
            );
            continue;
        };

        let time_scale = time.delta_secs();
        let cur_xz_forward = transform.forward().with_y(0.).normalize_or(Vec3::Z);
        let target_xz_forward = intent.heading.0; // x-z plane heading
        let linear_velocity = forces.linear_velocity();
        let angular_velocity = forces.angular_velocity();

        event.linear_velocity = linear_velocity;
        event.angular_velocity = angular_velocity;

        let pos = transform.translation();
        gizmos.arrow(pos, pos + cur_xz_forward, Srgba::BLUE);
        gizmos.arrow(pos, pos + target_xz_forward, Srgba::RED);

        if target_xz_forward.length() < 0.0001 {
            let angular_acceleration = -angular_velocity / time_scale;
            let linear_accelation = (-linear_velocity / time_scale).with_y(0.);


            // additional damping
            forces.apply_angular_acceleration(angular_acceleration);
            forces.apply_linear_acceleration(linear_accelation);
            if linear_velocity.length() < 0.0001 && angular_velocity.length() < 0.0001 {
                event.event = Some(RcEventType::StartStanding);
            } else {
                event.event = Some(RcEventType::StartStopping);
            }
            event.angular_acceleration = angular_acceleration;
            event.linear_acceleration = linear_accelation;
        } else {
            let speed_on_forward = linear_velocity.dot(cur_xz_forward);
            let speed_on_forward_forecast = speed_on_forward + rc_move.acceleration * time_scale;
            if speed_on_forward_forecast < rc_move.max_speed {
                let gap = rc_move.max_speed - speed_on_forward_forecast;
                let acc_needed = gap / time_scale;
                let linear_acceleration = cur_xz_forward * acc_needed;
                info!(
                    "{} linear acc: {:?} (max: {:?})",
                    rc_move.tag, acc_needed, rc_move.max_speed
                );
                forces.apply_linear_acceleration(linear_acceleration);
                event.linear_acceleration = linear_acceleration;

                if speed_on_forward < 0.0001 {
                    event.event = Some(RcEventType::StartMoving);
                }
            }

            let angular_gap = cur_xz_forward.angle_to(target_xz_forward, Vec3::Y);
            //let angular_gap_forecast = angular_gap + angular_velocity.y * time_scale;
            let angular_gap_forecast_t1 = angular_gap + angular_velocity.y * time_scale;
            let n_counter_stearing = 3;

            if angular_gap.signum() * angular_gap_forecast_t1.signum() < 0. {
                info!("reverse: stop steering");
                let vel = forces.angular_velocity_mut();
                vel.y *= 0.01;
                event.angular_velocity = *vel;
            } else {
                let vel = angular_velocity.y;
                let mut angular_gap_forecast_tn = angular_gap;
                let mut vel_forecast = vel;
                for _ in 0..n_counter_stearing {
                    angular_gap_forecast_tn += vel_forecast * time_scale;
                    let acc_needed = (angular_gap_forecast_tn / time_scale) - vel_forecast;
                    let acc_needed = acc_needed
                        .clamp(-rc_move.angular_acceleration, rc_move.angular_acceleration);
                    vel_forecast += acc_needed * time_scale;
                }

                if angular_gap.signum() * angular_gap_forecast_tn.signum() < 0. {
                    info!("reverse: counter steering");
                    let acc_needed = -vel / time_scale;
                    let angular_acceleration = Vec3::Y * acc_needed;
                    forces.apply_angular_acceleration(angular_acceleration);
                    event.angular_acceleration = angular_acceleration;
                } else {
                    let acc_needed =
                        (angular_gap_forecast_tn / (time_scale * n_counter_stearing as f32)) - vel;
                    let acc_needed = acc_needed
                        .clamp(-rc_move.angular_acceleration, rc_move.angular_acceleration);
                    let vel_forecast = vel + acc_needed * time_scale;
                    if vel_forecast.abs() > rc_move.max_angular_speed {
                        //forces.apply_angular_acceleration(Vec3::Y * -vel * 1.0 / time_scale);
                        //info!("slow down");
                    } else {
                        let angular_acceleration = Vec3::Y * acc_needed;
                        forces.apply_angular_acceleration(angular_acceleration);
                        event.angular_acceleration = angular_acceleration;
                    }
                }
            }
        }

        events.push(event);
    }

    for entity in remaining_entities {
        let Ok((_entity, forces, _transform, _rc_move)) = q_rc_move.get(entity) else {
            continue;
        };
        let linear_velocity = forces.linear_velocity();
        let angular_velocity = forces.angular_velocity();
        let event = UpdateRcMove {
            entity,
            event: None,
            linear_velocity,
            linear_acceleration: Vec3::ZERO,
            angular_velocity,
            angular_acceleration: Vec3::ZERO,
        };

        events.push(event);
    }

    for event in events {
        commands.entity(event.entity).trigger(|_e| event);
    }
}
