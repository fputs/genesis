# content

JSON fixtures and the loader the game (and a later editor) call to start a scenario. No Bevy dependency. This crate does not resolve checks or turns; it only reads and rejects data.

`load_encounter(path)` reads one encounter file. Creature and room paths inside it are relative to that file. The returned `EncounterFixture` always has four party slots.

## Glossary

| Field | Meaning |
| --- | --- |
| `hit_points` | Starting and maximum Hit Points |
| `armor_class` | Armor Class |
| `perception` | Perception modifier used for initiative |
| `speed_feet` | Speed in feet, a multiple of 5 |
| `strike.attack_bonus` | Modifier on the Strike's attack roll |
| `strike.damage` | `dice`, `faces`, and flat `bonus` |
| `strike.reach_feet` | Strike reach in feet |
| `cell_feet` | Square size; fixtures use 5 |
| `trigger.radius_feet` | Chebyshev distance in feet that starts combat |

Room rows use `#` for blocked and `.` for walkable. Unknown fields and failed checks return `LoadError` rather than a partial encounter.

The slice fixture is `fixtures/encounters/room_one.json`: one room, four level-1 fighters, one enemy, one proximity trigger.
