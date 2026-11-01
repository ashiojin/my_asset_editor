use avian3d::dynamics::rigid_body::forces::{Forces, WriteRigidBodyForces};
use bevy::prelude::*;

use crate::character_control::{
    CharacterController, JumpImpulse, intension::{PlayerIntention, PlayerIntentionType},
};

#[derive(Component, Debug, Clone, Default)]
pub struct JumpCapable {}

#[derive(EntityEvent, Debug)]
pub struct StartCrunting(Entity);

pub fn handle_jump_intentions(
    mut commands: Commands,
    mut q_jumper: Query<(Entity, &JumpCapable), With<CharacterController>>,
    mut player_indication_reader: MessageReader<PlayerIntention>,
) {
    for movement in player_indication_reader.read() {
        if !matches!(movement.action, PlayerIntentionType::Jump) {
            continue;
        }
        for (entity, _jump) in &mut q_jumper {
            commands.entity(entity).trigger(StartCrunting);
        }
    }
}


pub fn request_jump(mut entity: EntityWorldMut) {
    let Ok((jump_impulse, mut forces, _jump)) = entity.get_components_mut::<(&JumpImpulse, Forces, &JumpCapable)>()
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
