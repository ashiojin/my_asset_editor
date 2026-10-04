use bevy::{animation::AnimationEvent, prelude::*};

// /// A component that stores the description of animation graph events for an entity.
// ///
// /// It is handled by internal system automatically, and enables the user to get `ClipNodeEvent` as
// /// defined in the description.
// #[derive(Component, Debug)]
// #[require(NotYetExtacted<AnimationGraphEventsDesc>)]
// pub struct AnimationGraphEventsDesc(Vec<ClipNodeEventDefinitionDesc>);
// impl AnimationGraphEventsDesc {
//     pub fn new(events: Vec<ClipNodeEventDefinitionDesc>) -> Self {
//         Self(events)
//     }
// }
// impl From<&[ClipNodeEventDefinitionDesc]> for AnimationGraphEventsDesc {
//     fn from(events: &[ClipNodeEventDefinitionDesc]) -> Self {
//         Self(events.to_vec())
//     }
// }

/// An AnimationEvent that is triggered when a specific time in an animation clip is reached.
///
/// For the end user, we provide `ClipNodeEvent` which is triggered on the specific node of the
/// animation graph.
/// An AnimationClip can be shared by multiple AnimationPlayers && animation graph's nodes, so we
/// need to know which AnimationPlayer and which node of the animation graph triggered the event
/// and check if the event should be triggered or not. It is the reason why we need to store the
/// entity (AnimationPlayer) and the target node of the animation graph in this event.
#[derive(AnimationEvent, Debug, Clone)]
pub struct InnerClipEvent {
    event: String,
    entity: Entity,
    target_node: AnimationNodeIndex,
    time: f32,
}

impl InnerClipEvent {
    pub fn new(event: String, entity: Entity, target_node: AnimationNodeIndex, time: f32) -> Self {
        Self {
            event,
            entity,
            target_node,
            time,
        }
    }
    pub fn event_name(&self) -> &str {
        &self.event
    }
    pub fn entity(&self) -> Entity {
        self.entity
    }
    pub fn target_node(&self) -> AnimationNodeIndex {
        self.target_node
    }
    pub fn time(&self) -> f32 {
        self.time
    }
}
/// An event that is triggered when a specific time in an animation clip is reached.
#[derive(EntityEvent, Debug, Clone)]
pub struct ClipNodeEvent {
    entity: Entity,
    /// The name of the event that is triggered.
    /// It is originated `ClipNodeEventDefinitionDesc.event` (defined by the user) in the animation graph description.
    event: String,

    /// For debugging purpose, we store the original ClipEvent
    #[allow(dead_code)] // for debugging purpose
    source: InnerClipEvent,
}
impl ClipNodeEvent {
    pub fn new(entity: Entity, event: String, source: InnerClipEvent) -> Self {
        Self {
            entity,
            event,
            source,
        }
    }
    pub fn entity(&self) -> Entity {
        self.entity
    }
    pub fn event_name(&self) -> &str {
        &self.event
    }
}
