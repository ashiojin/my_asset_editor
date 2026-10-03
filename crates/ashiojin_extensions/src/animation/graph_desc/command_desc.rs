use bevy::prelude::*;

/// A collection of command descriptions for controlling the animation graph.
/// All commands in this collection will be executed in same frame
#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CommandsDesc{
    name: String,
    list: Vec<CommandDesc>,
}
impl CommandsDesc {
    pub fn new(name: String, list: Vec<CommandDesc>) -> Self {
        Self { name, list }
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn list(&self) -> &[CommandDesc] {
        &self.list
    }
}

/// A command description for controlling the animation graph.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum CommandDesc {
    Play(Play),
    Stop(Stop),
    SetWeight(SetWeight),
}

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Play {
    target_node: String,
    repeat: bool,
    speed: f32, // TODO: Consider using kind of a "Reference" type to allow the user to connect infomration from CharacterController and playing speed. e.g. walking speed to animation speed

}

impl Play {
    pub fn new(target_node: String, repeat: bool, speed: f32) -> Self {
        Self { target_node, repeat, speed }
    }
    pub fn from_target_node(target_node: String) -> Self {
        Self { target_node, repeat: false, speed: 1.0 }
    }
    pub fn with_repeat(mut self, repeat: bool) -> Self {
        self.repeat = repeat;
        self
    }
    pub fn with_speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }
    pub fn repeat(&self) -> bool {
        self.repeat
    }
    pub fn speed(&self) -> f32 {
        self.speed
    }
}

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Stop {
    target_node: String,
}

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SetWeight {
    target_node: String,
    /// [0, 1]
    weight: f32, // TODO: It has a same problem as Play.speed
}

impl SetWeight {
    pub fn new(target_node: String, weight: f32) -> Self {
        Self { target_node, weight }
    }
    pub fn weight(&self) -> f32 {
        self.weight
    }
}


pub trait HasCommandTarget {
    fn target_node(&self) -> &String;
}

impl HasCommandTarget for Play {
    fn target_node(&self) -> &String {
        &self.target_node
    }
}
impl HasCommandTarget for SetWeight {
    fn target_node(&self) -> &String {
        &self.target_node
    }
}
impl HasCommandTarget for Stop {
    fn target_node(&self) -> &String {
        &self.target_node
    }
}
impl HasCommandTarget for CommandDesc {
    fn target_node(&self) -> &String {
        match self {
            CommandDesc::Play(play) => play.target_node(),
            CommandDesc::Stop(stop) => stop.target_node(),
            CommandDesc::SetWeight(set_weight) => set_weight.target_node(),
        }
    }
}
