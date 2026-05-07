// Phase 5: MIDI input via midir.
//
// Planned:
//   MidiInput — lists available MIDI ports, opens selected port
//   Configurable mapping: CC number → fixture attribute, Note On → cue trigger
//   Callback forwarded to Bevy event queue via crossbeam_channel
