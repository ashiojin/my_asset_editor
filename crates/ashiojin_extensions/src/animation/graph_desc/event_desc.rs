use bevy::prelude::*;


///
/// Usecase of ClipNodeEventDefinitionDesc:
///   CC: CharacterController
///   AG: AnimationGraph
///   CT: Connector for CC and AG
///
///   In this crate we define:
///   - descripers for AG and AG-commands and AG-events.
///   - runtime instances and logic for AG and AG-commands and AG-events.
///
///   All the other parts are defined as a part of the CharacterController logic, which is out of the scope of this crate.
///
///   CC -[ev: StartCrunching] -> CT -[command: Play(ClipNode: Crunching)] -> AG
///    => AG plays the Crunching clip node
///   When the Crunching clip node reaches "jumping time"
///    => AG -[ev: Jumping]-> CT -[command: JumpAction]-> CC
///            A
///            |
///            * ClipNodeEventDefinitionDesc defines the event "Jumping" at "jumping time" in the Crunching clip node
///
#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ClipNodeEventDefinitionDesc {
    /// The name of the event that triggers this clip node.
    /// It should be unique within the animation graph.
    event: String,
    /// The name of the clip node that this event triggers.
    clipnode: String,
    /// The time in seconds at which the event should be triggered.
    time: f32,
}

impl ClipNodeEventDefinitionDesc {
    pub fn new(event: String, clipnode: String, time: f32) -> Self {
        Self { event, clipnode, time }
    }
    pub fn event(&self) -> &str {
        &self.event
    }
    pub fn clipnode(&self) -> &str {
        &self.clipnode
    }
    pub fn time(&self) -> f32 {
        self.time
    }
}

