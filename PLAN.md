# stageLX — Project Plan

A real-time 3D stage lighting visualizer and DMX controller written in Rust.
Supports GDTF fixture definitions, MVR scene files, and full DMX I/O.

---

## Goals

- Visualize medium-scale rigs (50–500 fixtures) at 60 fps on desktop hardware
- Parse GDTF fixture files from [gdtf-share.com](https://gdtf-share.com/) for accurate fixture capabilities and geometry
- Import and export MVR (My Virtual Rig) scenes for interoperability with MA3, Depence2, etc.
- Built-in programmer for standalone use, plus live DMX input from external consoles
- Full DMX output via Art-Net, sACN, and USB dongles (Enttec / DMXking)
- Accept control input from Art-Net/sACN, MIDI, and OSC

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                     stageLX (Bevy app)                  │
│                                                         │
│  ┌──────────────┐  ┌──────────────┐  ┌───────────────┐ │
│  │   3D Render  │  │  Programmer  │  │  Scene Editor │ │
│  │  (wgpu/Bevy) │  │  (egui)      │  │  (egui)       │ │
│  └──────┬───────┘  └──────┬───────┘  └───────┬───────┘ │
│         │                 │                  │         │
│  ┌──────▼─────────────────▼──────────────────▼───────┐ │
│  │                  Core ECS / Event Bus              │ │
│  │         (Bevy World — fixtures, patch, cues)       │ │
│  └──────┬────────────────────────────────────────────┘ │
│         │                                              │
│  ┌──────▼──────────────────────────────────────────┐  │
│  │               I/O Layer                         │  │
│  │  Art-Net Out │ sACN Out │ USB DMX │             │  │
│  │  Art-Net In  │ sACN In  │ MIDI In │ OSC In      │  │
│  └─────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
         ▲                              ▲
   GDTF / MVR files              Hardware / Network
```

---

## Cargo Workspace Structure

```
stageLX/
├── Cargo.toml                 # workspace root
├── PLAN.md
│
├── crates/
│   ├── stageLX-core/          # shared types: patch, universe, fixture instance
│   ├── stageLX-gdtf/          # GDTF + MVR parser
│   ├── stageLX-dmx/           # DMX frame engine + universe buffer
│   ├── stageLX-io/            # Art-Net, sACN, USB, MIDI, OSC
│   ├── stageLX-render/        # Bevy plugin: 3D scene, beams, gobos, fog
│   └── stageLX-ui/            # egui panels: patch, programmer, scene
│
└── src/
    └── main.rs                # Bevy App wiring
```

### Crate Responsibilities

| Crate | Responsibility |
|---|---|
| `stageLX-core` | `FixtureInstance`, `Patch`, `Universe`, `DmxBuffer`, attribute model |
| `stageLX-gdtf` | Parse `.gdtf` (ZIP+XML), geometry trees, DMX modes, wheels, physicals |
| `stageLX-dmx` | DMX frame generation, merge strategies (HTP/LTP), universe buffers |
| `stageLX-io` | Art-Net Tx/Rx, sACN Tx/Rx, USB serial (Enttec), MIDI, OSC |
| `stageLX-render` | Bevy plugin: volumetric beams, gobo projection, color, fog medium |
| `stageLX-ui` | egui: patch editor, programmer, fixture library, scene graph |

---

## Key Crate Dependencies

| Purpose | Crate |
|---|---|
| App/ECS/rendering | `bevy` (0.14+) |
| Low-level GPU | `wgpu` (via Bevy) |
| UI panels | `bevy_egui` + `egui` |
| GDTF/MVR ZIP parsing | `zip`, `quick-xml` |
| 3D model loading (GDTF geometry) | `bevy_gltf` or `obj` |
| Art-Net | `artnet-rs` or custom UDP |
| sACN (E1.31) | `sacn` crate or custom |
| USB/serial DMX | `serialport` |
| MIDI | `midir` |
| OSC | `rosc` |
| Async runtime | `tokio` (for I/O tasks, bridged to Bevy via channels) |
| Image loading (gobos) | `image` |

---

## GDTF Data Model (Key Concepts)

GDTF files are ZIP archives containing `description.xml` plus assets:

```
FixtureType
├── DMXModes[]           # e.g. "Basic 16ch", "Extended 32ch"
│   └── DMXChannels[]    # one per attribute instance
│       └── LogicalChannel → Attribute (Pan, Tilt, Dimmer, ColorAdd_R, Zoom…)
├── Geometries           # 3D hierarchy (Body → Yoke → Head for moving heads)
│   └── Beam             # defines beam angle, beam type, color temperature
├── Wheels[]             # ColorWheel, GoboWheel, EffectWheel
│   └── Slots[]          # individual colors/gobos with image assets
└── PhysicalDescriptions
    └── ColorRendering, Emitters, Filters
```

MVR files extend this with scene context:
- Fixture instances with 3D transform (position + rotation)
- Focus positions
- Truss / structure geometry
- Patch assignments

---

## Rendering Strategy

### Beams
- Render each beam as a **cone mesh** with additive blending
- Color = mixed output of ColorAdd_R/G/B channels (or color wheel slot color)
- Beam angle = Zoom attribute value mapped to `beam_angle` from GDTF physical
- Use **ray-marched volumetric cone shader** (wgpu custom shader) for fog effect
- Atmosphere/fog density controlled per-scene

### Gobos
- Load gobo images from GDTF wheel slot assets
- Project as texture via **projected spotlight** in Bevy PBR + custom shader
- Rotate with GoboIndex + GoboRotation attributes

### Fixture Geometry
- Load GDTF geometry from embedded 3DS/GLB/OBJ → convert to Bevy mesh
- Articulate geometry nodes: yoke rotates with Pan, head rotates with Tilt
- Fallback: procedural geometry (box for body, cylinder for head) when no model

### Color
- Additive CMY/RGB mixing in shader
- CIE color space for mixed colors when GDTF emitter data available
- Gel/filter colors from ColorWheel slot CIE values

---

## DMX Engine

- **Universe buffer**: `[u8; 512]` per universe, up to 64 universes
- **Merge**: HTP (Highest Takes Precedence) across input sources by default; configurable per universe
- **Priority**: Built-in programmer > Art-Net in > sACN in (configurable)
- **Output tick**: 44 Hz (standard DMX refresh), driven by Bevy fixed timestep
- **Attribute → DMX**: GDTF channel functions map raw DMX values to physical attribute values bidirectionally

---

## I/O Protocol Details

### Art-Net (Output + Input)
- UDP port 6454
- ArtDMX packet for output (universe 0–32767)
- ArtPoll/ArtPollReply for node discovery
- Run in dedicated `tokio` task, bridge to Bevy via `crossbeam-channel`

### sACN / E1.31 (Output + Input)
- UDP multicast or unicast
- Priority 100 default, configurable per universe
- Run in dedicated `tokio` task

### USB DMX (Output)
- Enttec DMX USB Pro protocol over serial (`serialport` crate)
- Single universe per device
- Auto-detect by USB VID/PID

### MIDI (Input)
- `midir` for cross-platform MIDI device access
- Configurable mapping: CC → attribute, Note → cue trigger
- Run in `midir` callback, forwarded to Bevy event queue

### OSC (Input)
- UDP, default port 8000
- Path schema: `/fixture/{id}/{attribute}` → float value
- `/cue/{id}/go` for cue triggers

---

## Implementation Phases

### Phase 1 — Foundation (Weeks 1–4)
**Goal**: A Bevy app that loads a GDTF file and renders placeholder fixtures in 3D.

- [ ] Cargo workspace scaffold
- [ ] `stageLX-gdtf`: parse GDTF ZIP + `description.xml` (fixture type, DMX modes, attributes)
- [ ] `stageLX-core`: `FixtureType`, `FixtureInstance`, `Patch`, `Universe`, `DmxBuffer`
- [ ] `stageLX-render`: Bevy plugin — procedural fixture geometry, static scene camera
- [ ] Basic egui panel: add fixture from library, set DMX address
- [ ] `stageLX-dmx`: universe buffer + attribute→DMX resolution from GDTF channel functions

**Milestone**: Patch 10 moving heads, manually set Pan/Tilt/Dimmer values, see fixtures move in 3D.

---

### Phase 2 — Programmer + Beam Rendering (Weeks 5–8)
**Goal**: Real-time beam visualisation driven by the built-in programmer.

- [ ] Programmer UI: attribute faders, color picker, gobo selector
- [ ] Selection model: select fixtures, record groups
- [ ] Beam cone mesh with additive blending shader
- [ ] Color mixing (RGB/CMY/ColorWheel) in shader
- [ ] Gobo projection (rotating gobo texture)
- [ ] Zoom/beam angle attribute driving cone geometry
- [ ] Strobe simulation (Shutter channel)
- [ ] Dimmer channel → beam intensity

**Milestone**: Full real-time beam visualization driven by programmer at 60 fps with 100 fixtures.

---

### Phase 3 — DMX I/O (Weeks 9–12)
**Goal**: Send and receive DMX from real hardware and consoles.

- [ ] `stageLX-io`: Art-Net output (ArtDMX packets)
- [ ] Art-Net input listener (receive from consoles like MA3, EOS)
- [ ] sACN output
- [ ] sACN input
- [ ] USB DMX output (Enttec USB Pro)
- [ ] HTP merge across input sources
- [ ] Universe/port configuration UI

**Milestone**: Receive Art-Net from a console, visualize result, simultaneously output Art-Net and USB DMX.

---

### Phase 4 — MVR + GDTF Geometry (Weeks 13–16)
**Goal**: Import a full show file from MA3 / Depence2 and see the venue + patch.

- [ ] `stageLX-gdtf`: MVR parser (scene positions, fixture placement, trusses)
- [ ] Import MVR: place fixtures in 3D from MVR transforms
- [ ] Load GDTF 3D geometry (OBJ/3DS → Bevy mesh)
- [ ] Articulated geometry: yoke/head rotation driven by Pan/Tilt
- [ ] MVR export (scene + patch → `.mvr` file)
- [ ] Truss / structure geometry from MVR

**Milestone**: Import an MVR file exported from MA3, see accurate venue layout and fixture models.

---

### Phase 5 — MIDI, OSC + Advanced Rendering (Weeks 17–20)
**Goal**: Full input surface coverage + professional rendering quality.

- [ ] MIDI input: CC → attribute mapping, configurable per fixture/group
- [ ] OSC input: `/fixture/{id}/{attr}` message handling
- [ ] Volumetric fog/atmosphere shader (ray-marched cone)
- [ ] Gobo physical rotation (indexed vs. rotating wheel behavior)
- [ ] Iris / shutter cuts (framing shutters if in GDTF)
- [ ] Multiple viewports (FOH view, top view, side view)
- [ ] Camera animation / saved positions

**Milestone**: Full live control from a MIDI surface, OSC from TouchDesigner, with volumetric beams in fog.

---

## Open Questions / Decisions Needed

1. **GDTF 3D model format**: GDTF v1.1 uses 3DS format for geometry. Need a 3DS loader or conversion pipeline (3DS → glTF at import time). Evaluate `three-d` or write a minimal 3DS loader.

2. **Bevy version**: Bevy 0.14 vs 0.15 — 0.15 has Required Components which maps well to fixture attributes. Track release schedule.

3. **Shader approach for beams**: Custom wgpu shader via Bevy's `Material` trait vs. `bevy_hanabi` particle system for beam dust. Custom shader gives more control for gobo projection.

4. **Cue system**: Not in scope for v1, but the data model should not foreclose adding a cue stack later. `DmxBuffer` should support named snapshots.

5. **GDTF-share API**: gdtf-share.com has a REST API for downloading fixture files by manufacturer/model. Worth integrating a fixture browser that can pull directly from the share.

6. **Test strategy**: GDTF files vary wildly in quality. Build a fixture file test corpus (grab 20–30 files from gdtf-share.com across manufacturers) and validate parser against them early.

---

## Non-Goals (v1)

- Cue stack / show playback (Phase 2+ of the project, after v1 ships)
- Video / media server integration
- Path-traced / offline rendering
- Mobile or web targets
- Network multi-user collaboration

---

## Repository Setup

```bash
git init
git remote add origin https://github.com/BlueJayLouche/stageLX.git
cargo init --name stageLX
# then convert to workspace and add crates/
```

Suggested `.gitignore`: standard Rust gitignore + `*.gdtf` test files (large binaries).

---

*Last updated: 2026-05-07*
