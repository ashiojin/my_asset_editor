use bevy::prelude::*;

mod trail;

pub struct EmitterPlugin;

impl Plugin for EmitterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (trail::apply_sandbox_fx_meshes, trail::spawn_trail_from_emitter));
    }
}
