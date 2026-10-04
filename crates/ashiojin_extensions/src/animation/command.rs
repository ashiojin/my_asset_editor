use bevy::{platform::collections::HashMap, prelude::*};

use crate::animation::LinkToAnimationPlayer;

#[derive(Debug, Clone)]
pub enum Command {
    Play(Play),
    Stop(Stop),
    SetWeight(SetWeight),
}

#[derive(Debug, Clone)]
pub struct Play {
    target_node: AnimationNodeIndex,
    repeat: bool,
    speed: f32,
}

impl Play {
    pub fn new(target_node: AnimationNodeIndex, repeat: bool, speed: f32) -> Self {
        Self {
            target_node,
            repeat,
            speed,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Stop {
    target_node: AnimationNodeIndex,
}

impl Stop {
    pub fn new(target_node: AnimationNodeIndex) -> Self {
        Self { target_node }
    }
}

#[derive(Debug, Clone)]
pub struct SetWeight {
    target_node: AnimationNodeIndex,
    weight: f32, // TODO: It has a same problem as Play.speed
}

impl SetWeight {
    pub fn new(target_node: AnimationNodeIndex, weight: f32) -> Self {
        Self {
            target_node,
            weight,
        }
    }
}

/// A component that stores the description of animation graph commands for an entity.
///
/// It is handled by internal system automatically, and enables the user to emit `AnimationGraphCommandRequest` to the entity to control the animation graph.
// #[derive(Component, Debug)]
// #[require(NotYetExtacted<AnimationGraphCommandsDesc>)]
// pub struct AnimationGraphCommandsDesc(Vec<CommandsDesc>);

#[derive(Component, Debug)]
pub struct DefinedAnimationGraphCommands(HashMap<String, Vec<Command>>);
impl DefinedAnimationGraphCommands {
    pub fn new(defined_commands: HashMap<String, Vec<Command>>) -> Self {
        Self(defined_commands)
    }
}

pub struct AnimationGraphCommandRequest(pub String);

impl EntityCommand for AnimationGraphCommandRequest {
    type Out = ();
    fn apply(self, mut entity: EntityWorldMut) -> Self::Out {
        enum PlayerCmd {
            Play(Play),
            Stop(Stop),
        }
        let id = entity.id();
        let (player_commands, graph_commands) = {
            let Some(defined_commands) = entity
                .get_components::<&DefinedAnimationGraphCommands>()
                .ok()
            else {
                error!(
                    "entity {:?} does not have DefinedAnimationGraphCommands component",
                    id
                );
                return;
            };

            let Some(commands) = defined_commands.0.get(&self.0) else {
                error!("entity {:?} does not have commands for {}", id, self.0);
                return;
            };

            let player_commands = commands
                .iter()
                .filter_map(|c| match c {
                    Command::Play(play) => Some(PlayerCmd::Play(play.clone())),
                    Command::Stop(stop) => Some(PlayerCmd::Stop(stop.clone())),
                    Command::SetWeight(_) => None,
                })
                .collect::<Vec<_>>();
            let graph_commands = commands
                .iter()
                .filter_map(|c| match c {
                    Command::Play(_) => None,
                    Command::Stop(_) => None,
                    Command::SetWeight(set_weight) => Some(set_weight.clone()),
                })
                .collect::<Vec<_>>();

            (player_commands, graph_commands)
        };
        let player_entity = {
            let Some(link_to_player) = entity.get_components::<&LinkToAnimationPlayer>().ok()
            else {
                error!(
                    "entity {:?} does not have LinkToAnimationPlayer component",
                    id
                );
                return;
            };
            link_to_player.player_entity()
        };

        {
            let Some(graph_handle) = entity.world_scope(|world| {
                let entity = world.entity(player_entity);
                entity.get_components::<&AnimationGraphHandle>().cloned().ok()
            }) else {
                error!(
                    "entity {:?} does not have AnimationGraphHandle component",
                    id
                );
                return;
            };


            // let Ok(graph_handle) = entity.get_components::<&AnimationGraphHandle>().cloned() else {
            //     error!(
            //         "entity {:?} does not have AnimationGraphHandle component",
            //         id
            //     );
            //     return;
            // };

            let Some(mut graphs) = entity.get_resource_mut::<Assets<AnimationGraph>>() else {
                error!(
                    "entity {:?} does not have AnimationGraphResource resource",
                    id
                );
                return;
            };

            let Some(mut graph) = graphs.get_mut(graph_handle.id()) else {
                error!(
                    "entity {:?} does not have AnimationGraph for handle {:?}",
                    id, graph_handle
                );
                return;
            };

            for command in graph_commands {
                let Some(node) = graph.get_mut(command.target_node) else {
                    error!(
                        "entity {:?} does not have node {:?} in animation graph",
                        id, command.target_node
                    );
                    return;
                };
                node.weight = command.weight;
            }
        }

        entity.world_scope(|world| {
            let mut player_entity = world.entity_mut(player_entity);
            let Some(mut player) = player_entity
                .get_components_mut::<&mut AnimationPlayer>()
                .ok()
            else {
                error!(
                    "entity {:?} does not have AnimationGraphPlayer component",
                    player_entity.id()
                );
                return;
            };

            for command in player_commands {
                match command {
                    PlayerCmd::Play(play) => {
                        let mut p = player.start(play.target_node).set_speed(play.speed);
                        if play.repeat {
                            p = p.repeat();
                        }
                    }
                    PlayerCmd::Stop(stop) => {
                        player.stop(stop.target_node);
                    }
                }
            }
        });
    }
}
