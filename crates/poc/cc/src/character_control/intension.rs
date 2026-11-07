use bevy::prelude::*;

#[derive(Message, Debug)]
pub struct PlayerIntention {
    pub jump: bool,
    pub heading: Heading,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Heading(pub Vec3);

