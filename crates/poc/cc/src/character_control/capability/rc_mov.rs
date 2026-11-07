use std::{f32::consts::PI, rc};

use avian3d::dynamics::rigid_body::{AngularVelocity, LinearVelocity, forces::{Forces, ReadRigidBodyForces, WriteRigidBodyForces}};
use bevy::prelude::*;

use crate::character_control::{
    CharacterController, intension::{PlayerIntention},
};

#[derive(Component, Debug, Clone, Default)]
pub struct RcMoveCapable {
    pub max_speed: f32,
    pub acceleration: f32,

    pub max_angular_speed: f32,
    pub angular_acceleration: f32,
}

pub fn handle_rc_move_intentions(
    //mut commands: Commands,
    mut q_rc_move: Query<(Entity, Forces, &GlobalTransform, &RcMoveCapable), With<CharacterController>>,
    mut player_indication_reader: MessageReader<PlayerIntention>,
    time: Res<Time>,
    mut gizmos: Gizmos
) {
    for intent in player_indication_reader.read() {
        for (_entity, mut forces, transform, rc_move) in &mut q_rc_move {
            let time_scale = time.delta_secs();
            let cur_xz_forward = transform.forward().with_y(0.).normalize_or(Vec3::Z);
            let target_xz_forward = intent.heading.0; // x-z plane heading
            let linear_velocity = forces.linear_velocity();
            let angular_velocity = forces.angular_velocity();

            let pos = transform.translation();
            gizmos.arrow(pos, pos + cur_xz_forward, Srgba::BLUE);
            gizmos.arrow(pos, pos + target_xz_forward, Srgba::RED);

            if target_xz_forward.length() < 0.0001 {
                // additional damping
                forces.apply_angular_acceleration(-angular_velocity * 1.0);
                forces.apply_linear_acceleration(-linear_velocity * 1.0);
                continue;
            }

            // FIXME: It does not work collectly almost all the time
            let speed_on_forward = linear_velocity.dot(cur_xz_forward);
            let speed_on_forward_forecast = speed_on_forward + rc_move.acceleration * time_scale;
            if speed_on_forward_forecast < rc_move.max_speed {
                let gap = rc_move.max_speed - speed_on_forward_forecast;
                let acc_needed = gap / time_scale;
                forces.apply_linear_acceleration(cur_xz_forward * acc_needed);
            }
            // if speed_on_forward < rc_move.max_speed {
            //     forces.apply_linear_acceleration(cur_xz_forward * rc_move.acceleration);
            // }

            let angular_gap = cur_xz_forward.angle_to(target_xz_forward, Vec3::Y);
            //let angular_gap_forecast = angular_gap + angular_velocity.y * time_scale;
            let n = 5.;
            let angular_gap_forecast_t_x = angular_gap + angular_velocity.y * (time_scale * n);
            if angular_gap_forecast_t_x.abs() > PI * 0.01 {
                let vel = angular_velocity.y;
                let acc_needed = (angular_gap_forecast_t_x / time_scale) - vel;
                let acc_needed = acc_needed.clamp(-rc_move.angular_acceleration, rc_move.angular_acceleration);
                let vel_forecast = vel + acc_needed * time_scale * n;
                if vel_forecast.abs() > rc_move.max_angular_speed {
                    //forces.apply_angular_acceleration(Vec3::Y * -vel * 1.0 / time_scale);
                    //info!("slow down");
                } else {
                    forces.apply_angular_acceleration(Vec3::Y * acc_needed);
                    info!("angular_gap: {}, vel: {}, acc_needed: {}, vel_forecast: {}", angular_gap, vel, acc_needed, vel_forecast);
                }
            } else {
                info!("it is already aligned");
                forces.apply_angular_acceleration(-angular_velocity * 1.0 / time_scale);
            }




            // if angular_velocity.y.abs() < rc_move.max_angular_speed {
            //     forces.apply_angular_acceleration(Vec3::Y * angular_gap.signum() * rc_move.angular_acceleration);
            // }

            // info!("L: {:?}, A: {:?}, cur_F: {:?}, target_F: {:?}, applyed: L: {:?}/{:?}, A: {:?}/{:?}",
            //     linear_velocity, angular_velocity, cur_xz_forward, target_xz_forward,
            //     speed_on_forward < rc_move.max_speed, target_xz_forward * rc_move.acceleration,
            //     angular_velocity.y.abs() < rc_move.max_angular_speed, Vec3::Y * angular_gap.signum() * rc_move.angular_acceleration
            // );
        }
    }
}

