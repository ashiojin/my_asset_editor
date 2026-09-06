# TODO

## Graph features

- [ ] Macro
- [ ] Additional layer that abstracts animation graph operations
- [ ] Character Controller


### Macro

```
=== Graph ===
@Graph editor

 +--------SELECT-----------+
 |                         |
 | ClipName[0]:Walk        |
 |                         O--> [Blend W=1.0] --> ...
 | ClipName[1]:Slash       |
 |                         |
 +-------------------------+
 
   ↓
@Graph data
[Clip1 Clip=Walk  W=1.0] -----> O --> [Blend W=1.0] --> ...
                                A
                                |
[Clip2 Clip=Slash W=0.0] -------+

=== Operation ===

@Graph editor
SELECT::PlayOne 1
   ↓
@Graph operations
SetWeight "Clip1" 0.0
Stop      "Clip1"
SetWeight "Clip2" 1.0
Play      "Clip2"
(+ Remenber "Clip2" as "SELECT:Current"

@Graph editor
// after PlayOne "Clip1"
SELECT::PlayTransition 0
   ↓
@Graph operations
Play      "Clip1"
(+ Remenber "Clip2(=@SELECT:Current)" as "SELECT:From" and "Clip1" as "SELECT:To"

@Graph editor
// after SELECT::PlayTransition 0
SELECT::SetWeightTransition 0.25
   ↓
@Graph operations
SetWeight "@From" 0.75
SetWeight "@To" 0.25
```

### Layer that abstracts animation graph operations


```
[Animation Graph] ---EVENTS--> [State machine] ---OPERATIONS--> [Animation Graph]
```

EVENTS:
- Played("Clip1", "SlashEnd") : "Clip1" node is now played over "SlashEnd" point


