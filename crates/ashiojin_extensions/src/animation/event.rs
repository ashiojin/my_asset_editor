use bevy::animation::{AnimationEvent, RepeatAnimation};
use bevy::prelude::*;

use crate::animation::AnimationGraphHelper;
use crate::animation::graph_desc::event_desc::ClipNodeEventDefinitionDesc;
use crate::common::NotYetExtacted;

/// A component that stores the description of animation graph events for an entity.
///
/// It is handled by internal system automatically, and enables the user to get `ClipNodeEvent` as
/// defined in the description.
#[derive(Component, Debug)]
#[require(NotYetExtacted<AnimationGraphEventsDesc>)]
pub struct AnimationGraphEventsDesc(Vec<ClipNodeEventDefinitionDesc>);
impl AnimationGraphEventsDesc {
    pub fn new(events: Vec<ClipNodeEventDefinitionDesc>) -> Self {
        Self(events)
    }
}
impl From<&[ClipNodeEventDefinitionDesc]> for AnimationGraphEventsDesc {
    fn from(events: &[ClipNodeEventDefinitionDesc]) -> Self {
        Self(events.to_vec())
    }
}

/// An AnimationEvent that is triggered when a specific time in an animation clip is reached.
///
/// For the end user, we provide `ClipNodeEvent` which is triggered on the specific node of the
/// animation graph.
/// An AnimationClip can be shared by multiple AnimationPlayers && animation graph's nodes, so we
/// need to know which AnimationPlayer and which node of the animation graph triggered the event
/// and check if the event should be triggered or not. It is the reason why we need to store the
/// entity (AnimationPlayer) and the target node of the animation graph in this event.
#[derive(AnimationEvent, Debug, Clone)]
struct ClipEvent {
    event: String,
    entity: Entity,
    target_node: AnimationNodeIndex,
    time: f32,
}

impl ClipEvent {
    fn new(event: String, entity: Entity, target_node: AnimationNodeIndex, time: f32) -> Self {
        Self {
            event,
            entity,
            target_node,
            time,
        }
    }
    fn event_name(&self) -> &str {
        &self.event
    }
    fn entity(&self) -> Entity {
        self.entity
    }
    fn target_node(&self) -> AnimationNodeIndex {
        self.target_node
    }
    fn time(&self) -> f32 {
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
    source: ClipEvent,
}
impl ClipNodeEvent {
    fn new(entity: Entity, event: String, source: ClipEvent) -> Self {
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

pub fn apply_anim_graph_events_desc(
    mut commands: Commands,
    q_not_yet: Query<
        (Entity, &AnimationGraphHelper, &AnimationGraphEventsDesc),
        With<NotYetExtacted<AnimationGraphEventsDesc>>,
    >,
    mut animatnion_clips: ResMut<Assets<AnimationClip>>,
) {
    for (entity, graph_helper, events_desc) in &q_not_yet {
        for event_desc in &events_desc.0 {
            let node_idx = graph_helper
                .node_id_to_idx()
                .get(event_desc.clipnode())
                .unwrap_or_else(|| {
                    panic!(
                        "target node {} (in {}) not found in animation graph",
                        event_desc.clipnode(),
                        event_desc.event()
                    )
                });
            let h_clip = graph_helper
                .node_idx_to_clip_handle()
                .get(node_idx)
                .unwrap_or_else(|| {
                    panic!(
                        "clip handle for node {} (in {}) not found in animation graph",
                        event_desc.clipnode(),
                        event_desc.event()
                    )
                });
            let mut clip = animatnion_clips.get_mut(h_clip).unwrap_or_else(|| {
                panic!(
                    "clip for node {} (in {}) not found in animation graph",
                    event_desc.clipnode(),
                    event_desc.event()
                )
            });
            clip.add_event(
                event_desc.time(),
                ClipEvent::new(
                    event_desc.event().to_string(),
                    entity,
                    *node_idx,
                    event_desc.time(),
                ),
            );
            debug!(
                "added event {} at time {} for node {} (entity {:?})",
                event_desc.event(),
                event_desc.time(),
                event_desc.clipnode(),
                entity
            );
        }
        commands.add_observer(
            |trigger: On<ClipEvent>,
             mut commands: Commands,
             q_player: Query<(Entity, &AnimationPlayer)>| {
                 let Ok((_entity, player)) = q_player.get(trigger.entity) else {
                     warn!(
                         "entity {:?} does not have AnimationPlayer component",
                         trigger.entity
                     );
                     return;
                 };

                 let animation =  player.animation(trigger.target_node())
                     .unwrap_or_else(|| {
                         panic!(
                             "animation for node {:?} not found in AnimationPlayer for entity {:?}",
                             trigger.target_node(),
                             trigger.entity()
                         )
                     });

                 let last_seek_time = animation.last_seek_time();
                 let seek_time = animation.seek_time();
                 let repeat = animation.repeat_mode() != RepeatAnimation::Never;
                 let completions = animation.completions();

                 // patterns:
                 // - completions > 1: it means the animation has completed at least once, so we can trigger the event
                 // - completions == 1 && repeat: we should check [last_seek_time.or(0), duration] and [0, seek_time] to see if the event time is in either range
                 // - completions == 1 && !repeat: we should check [last_seek_time.or(0), duration] to see if the event time is in that range
                 // - completions == 0: we should check [last_seek_time.or(0), seek_time] to see if the event time is in that range

                 let should_trigger = if completions > 1 {
                     true
                 } else if completions == 1 {
                     if repeat {
                         (last_seek_time.is_none_or(|lst| lst < trigger.time())
                             && trigger.time() <= seek_time)
                             || (trigger.time() <= seek_time
                                 || last_seek_time.is_some_and(|lst| lst < trigger.time()))
                     } else {
                         last_seek_time.is_none_or(|lst| lst < trigger.time())
                             && trigger.time() <= seek_time
                     }
                 } else {
                     last_seek_time.is_none_or(|lst| lst < trigger.time())
                         && trigger.time() <= seek_time
                 };
                 if should_trigger {
                     debug!(
                         "triggering event {} for entity {:?} at time {} (last_seek_time: {:?}, seek_time: {}, completions: {}, repeat: {})",
                         trigger.event_name(),
                         trigger.entity,
                         trigger.time(),
                         last_seek_time,
                         seek_time,
                         completions,
                         repeat
                     );
                     commands.entity(trigger.entity).trigger(|e| {
                         ClipNodeEvent::new(e, trigger.event_name().to_string(), trigger.clone())
                     });
                 }
             },
        );

        commands
            .entity(entity)
            .try_remove::<NotYetExtacted<AnimationGraphEventsDesc>>();
    }
}
