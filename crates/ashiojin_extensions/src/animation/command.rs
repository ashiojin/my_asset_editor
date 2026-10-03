use bevy::{platform::collections::HashMap, prelude::*};

use crate::animation::AnimationGraphHelper;
use crate::animation::graph_desc::command_desc::{CommandsDesc, HasCommandTarget};
use crate::common::NotYetExtacted;

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

#[derive(Debug, Clone)]
pub struct Stop {
    target_node: AnimationNodeIndex,
}

#[derive(Debug, Clone)]
pub struct SetWeight {
    target_node: AnimationNodeIndex,
    weight: f32, // TODO: It has a same problem as Play.speed
}

/// A component that stores the description of animation graph commands for an entity.
///
/// It is handled by internal system automatically, and enables the user to emit `AnimationGraphCommandRequest` to the entity to control the animation graph.
#[derive(Component, Debug)]
#[require(NotYetExtacted<AnimationGraphCommandsDesc>)]
pub struct AnimationGraphCommandsDesc(Vec<CommandsDesc>);

#[derive(Component, Debug)]
pub struct DefinedAnimationGraphCommands(HashMap<String, Vec<Command>>);

pub fn apply_anim_graph_commands_desc(
    mut commands: Commands,
    q_not_yet: Query<
        (Entity, &AnimationGraphCommandsDesc, &AnimationGraphHelper),
        With<NotYetExtacted<AnimationGraphCommandsDesc>>,
    >,
) {
    use crate::animation::graph_desc::command_desc;
    for (entity, desc, helper) in &q_not_yet {
        let mut defined_commands = HashMap::new();
        for desc in &desc.0 {
            let mut commands = Vec::new();
            for command_desc in desc.list() {
                let node_idx = *helper
                    .node_id_to_idx()
                    .get(command_desc.target_node())
                    .unwrap_or_else(|| {
                        panic!(
                            "target node {} (in {}) not found in animation graph",
                            command_desc.target_node(),
                            desc.name()
                        )
                    });

                let command = match command_desc {
                    command_desc::CommandDesc::Play(play) => Command::Play(Play {
                        target_node: node_idx,
                        repeat: play.repeat(),
                        speed: play.speed(),
                    }),
                    command_desc::CommandDesc::Stop(_stop) => Command::Stop(Stop {
                        target_node: node_idx,
                    }),
                    command_desc::CommandDesc::SetWeight(set_weight) => {
                        Command::SetWeight(SetWeight {
                            target_node: node_idx,
                            weight: set_weight.weight(),
                        })
                    }
                };
                commands.push(command);
            }

            defined_commands.insert(desc.name().to_string(), commands);
        }

        commands
            .entity(entity)
            .try_insert(DefinedAnimationGraphCommands(defined_commands))
            .try_remove::<NotYetExtacted<AnimationGraphCommandsDesc>>();
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

        {
            let Ok(graph_handle) = entity.get_components::<&AnimationGraphHandle>().cloned() else {
                error!(
                    "entity {:?} does not have AnimationGraphHandle component",
                    id
                );
                return;
            };

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

        {
            let Some(mut player) = entity.get_components_mut::<&mut AnimationPlayer>().ok() else {
                error!(
                    "entity {:?} does not have AnimationGraphPlayer component",
                    id
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
        }
    }
}
