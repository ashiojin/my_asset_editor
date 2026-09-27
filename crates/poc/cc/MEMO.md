

## For the Next

### Spawning

- Transforms of Collider and ShapeCaster for the ground should be specfied
  in Gltf extensions or our asset file having Gltf path & animation graph
  (or graph file path).
- Binding infomation from CharacterController events to AnimationGraph commands
  likes "If it lands heavily, play 'landing-heave' node and set the node's weight
  to 1.0, and some other nodes's weights to 0.0".
- Loading state cheker required.

### Initializing

- Any CharacterController actions that must be emitted from an animation graph should
  be defined within the graph itself.
  e.g. "crunting" clip node: When the animation reaches a jumping timing,
       the graph emits "JumpImpulse" action

