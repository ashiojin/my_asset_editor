use bevy::prelude::*;


#[derive(Message, Debug)]
pub struct PlayerIntention {
    // TODO: Entity to specify which character is intended to perform the action
    pub action: PlayerIntentionType,
}

#[derive(Debug, Clone)]
pub enum PlayerIntentionType {
    Jump,
}
