// Phase 1/2: fixture geometry in the 3D scene.
//
// Plan:
//   FixtureVisual — Bevy component marking an entity as a rendered fixture
//   spawn_fixture_visual() — creates procedural body+yoke+head mesh hierarchy
//   articulate_fixtures() — system: reads Pan/Tilt channel values → rotates yoke/head transforms
//   Phase 4: swap procedural geometry for GDTF-embedded 3D models (OBJ/3DS → glTF)
