use avian3d::dynamics::rigid_body::forces::{Forces, WriteRigidBodyForces};
use bevy::prelude::*;

use crate::character_control::{
    CharacterController, JumpImpulse, intension::{PlayerIntention},
};

#[derive(Component, Debug, Clone, Default)]
pub struct JumpCapable {
    tag: &'static str,
}

impl JumpCapable {
    pub fn new(tag: &'static str) -> Self {
        Self { tag }
    }
}

#[derive(EntityEvent, Debug)]
pub struct StartCrunting(Entity);

pub fn handle_jump_intentions(
    mut commands: Commands,
    mut q_jumper: Query<(Entity, &JumpCapable), With<CharacterController>>,
    mut player_indication_reader: MessageReader<PlayerIntention>,
) {
    for intention in player_indication_reader.read() {
        if !intention.jump {
            continue;
        }
        if let Ok((_entity, jump)) = q_jumper.get(intention.entity) {
            debug!("Jump intention received for entity {:?} {}", intention.entity, jump.tag);
            commands.entity(intention.entity).trigger(StartCrunting);
        } else {
            warn!(
                "entity {:?} does not have JumpCapable component",
                intention.entity
            );
        }
    }
}


pub fn request_jump(mut entity: EntityWorldMut) {
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
