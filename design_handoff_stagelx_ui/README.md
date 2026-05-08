# Handoff: stageLX — UI Redesign

## Overview

stageLX is a stage-lighting control application built with Rust + Bevy + bevy_egui. The current `stagelx-ui` crate ships four floating egui windows (Programmer, Patch, Fixture Library, DMX I/O) layered on top of three 3D viewports (FOH + TOP + SIDE).

This handoff is a **redesign of all four panels and the overall layout**, moving from overlapping floating windows to a docked hybrid console layout with a more disciplined visual system. It also tightens information density, replaces saturated multi-color section headers with a restrained two-accent system, and upgrades the most important controls (encoders, faders, swatch grids) from generic egui widgets to purpose-built console controls.

## About the Design Files

The files under `prototype/` are **design references created in HTML/React** — high-fidelity mockups showing the intended look, layout, and behavior. **They are not production code to ship.** The implementation target for stageLX is **Rust + Bevy + bevy_egui**, the framework already in use; the task is to recreate these designs there using egui's `Painter` / `Response` APIs and the patterns established in the existing crate.

If you are building a non-egui port (web, native, etc.), use these designs as the visual spec and apply your stack's idioms — the React code is illustrative only.

The original Rust source (the four `*.rs` files this redesign replaces) is included under `source-reference/` for context — preserve the existing public API (`StageLxUiPlugin`, the `Res*` resources, the `Spawn/DespawnFixtureEvent` events, etc.) so the rest of the workspace keeps working.

## Fidelity

**High-fidelity (hifi).** All colors, type sizes, weights, spacing, radii, control shapes, and layouts are final. Implement pixel-perfectly. Where egui can't natively reproduce a control (e.g., the circular encoder with center readout), build a custom widget rather than approximating with a slider.

## Files in this bundle

```
prototype/
  index.html          ← entry point: open this to see the full design canvas
  tokens.css          ← design tokens (colors, type, spacing, radii, shadows)
  design-canvas.jsx   ← canvas host (pan/zoom, focus mode)
  atoms.jsx           ← Panel, Section, Encoder, Fader, Swatch, Tab, Icon
  programmer.jsx      ← Programmer panel
  patch.jsx           ← Patch panel
  library.jsx         ← Library panel (Fixtures / MVR / Venue tabs)
  io.jsx              ← DMX I/O panel
  layout.jsx          ← Full hybrid-dock layout: top bar, viewports, status bar
source-reference/
  lib.rs, programmer.rs, patch.rs, library.rs, io_panel.rs   ← original code
```

To preview: open `prototype/index.html` in a browser.

---

## Layout philosophy — Hybrid dock

The application chrome divides the window into five fixed regions plus a status bar. Panels are docked by default; any panel can detach to a floating egui window (preserving the current `egui::Window` code path as the detached state).

```
┌──────────────────────────────────────────────────────────────────┐
│  TOP BAR · 36 px                                                  │
├──────────┬──────────────────────────────┬───────────────────────┤
│          │   FOH viewport      ┌─ TOP ─┤                        │
│  LEFT    │   (3D perspective)  │       │   RIGHT RAIL          │
│  RAIL    │                     ├──────┤   320 px              │
│  300 px  │                     │ SIDE │                        │
│          │                     │      │   DMX I/O             │
│  Prog-   ├─────────────────────┴──────┤                        │
│  rammer  │  PATCH (docked)  │ LIBRARY │                        │
│          │  248 px tall     │ 248 px  │                        │
├──────────┴──────────────────────────────┴───────────────────────┤
│  STATUS BAR · 22 px                                               │
└──────────────────────────────────────────────────────────────────┘
```

- **Left rail (300 px)**: Programmer. Detachable.
- **Center top**: viewports — FOH (3 cols) plus stacked TOP / SIDE (1 col). Hairline 1 px borders separate them; replaces the hand-drawn `egui::layer_painter` lines in `lib.rs`.
- **Center bottom (248 px)**: Patch and Library side by side at 1.4:1 ratio.
- **Right rail (320 px)**: DMX I/O.
- **Status bar (22 px)**: selection, patch totals, universe loads, venue, transport state.

Total reference window: **1440 × 880**.

---

## Design tokens

Implement once as a Rust module (e.g., `theme.rs`) and reference everywhere. All neutrals stay below 0.02 chroma; only accents and semantic colors carry hue.

### Surfaces (oklch)

| Token | oklch | Use |
|---|---|---|
| `bg-app` | `oklch(0.135 0.004 240)` | Window background, viewport letterbox |
| `bg-chrome` | `oklch(0.165 0.004 240)` | Top bar, rails, panel titlebars |
| `bg-panel` | `oklch(0.190 0.005 240)` | Panel bodies |
| `bg-raised` | `oklch(0.225 0.005 240)` | Buttons, selected rows, tab fills |
| `bg-input` | `oklch(0.150 0.004 240)` | Sunken inputs, list backgrounds |
| `bg-hover` | `oklch(0.245 0.005 240)` | Hover states |

### Borders

| Token | oklch | Use |
|---|---|---|
| `border` | `oklch(0.295 0.005 240)` | Panel borders, separators |
| `border-soft` | `oklch(0.245 0.005 240)` | Internal dividers, input outlines |
| `border-strong` | `oklch(0.385 0.005 240)` | Hovered borders, dashed dropzones |

### Text

| Token | oklch | Use |
|---|---|---|
| `fg` | `oklch(0.940 0.005 240)` | Primary text |
| `fg-secondary` | `oklch(0.700 0.005 240)` | Secondary labels |
| `fg-muted` | `oklch(0.520 0.005 240)` | Hints, placeholders |
| `fg-faint` | `oklch(0.400 0.005 240)` | Tertiary metadata |

### Accents (semantic only — never decorative)

| Token | oklch | Use |
|---|---|---|
| `accent` | `oklch(0.78 0.13 215)` | Primary — selection, active tab, encoder fill, TX |
| `accent-dim` | `oklch(0.60 0.10 215)` | Secondary accent, muted active states |
| `accent-bg` | `oklch(0.30 0.06 215)` | Selected row tint, primary button fill |
| `warning` | `oklch(0.80 0.12 80)` | Amber — warnings, strobe accent |
| `error` | `oklch(0.66 0.18 28)` | Red — errors only |
| `rx` (live) | `oklch(0.80 0.13 150)` | Green — receive, healthy, online |
| `idle` | `oklch(0.50 0.005 240)` | Gray — disabled / offline |

> **Discipline:** previously every section header (DIMMER, POSITION, BEAM, COLOUR, GOBO, ART-NET, sACN, USB DMX, MIDI, OSC) had its own saturated color. **All section headers in the redesign are neutral `fg-muted`.** Color is reserved for state.

### Typography

- **Sans:** IBM Plex Sans (weights 400, 500, 600). Fallback: `-apple-system, "Segoe UI", sans-serif`.
- **Mono:** IBM Plex Mono (weights 400, 500). Fallback: `ui-monospace`.
- Numbers/addresses always mono. Labels and prose always sans.
- Feature settings: `"ss01"` and `"cv11"` on sans, `"zero"` and `"ss02"` on mono (slashed zero, alternate digits).

| Style | Size | Weight | Tracking | Use |
|---|---|---|---|---|
| Wordmark | 14 px | 600 | -0.01em | Top bar "stageLX" |
| Encoder readout | 18 px | 500 mono | -0.02em | Center of dial |
| Fader readout | 14 px | 500 mono | -0.01em | Above fader |
| Big counter | 16 px | 400 mono | -0.01em | I/O TX/RX numbers |
| Panel title | 11 px | 600 | 0.04em | Titlebar |
| Mode tab | 11 px | 600 (active) / 500 | 0.02em | Top-bar mode tabs |
| Body row | 11 px | 400 / 500 | 0 | Patch rows, fixture names |
| Address mono | 11 px | 400 mono | 0 | `1.001` style |
| Field label | 10 px | 500 | 0.02em UPPER | Form labels |
| Eyebrow | 9 px | 500 mono | 0.14em UPPER | Section headers (POSITION, COLOUR…) |
| Hint | 9 px | 400 mono | 0 | Secondary metadata |

### Spacing

4 px grid. Common values: 4, 6, 8, 10, 12, 14, 24, 28.

### Radii

| Token | px | Use |
|---|---|---|
| `r-1` | 2 | Inner controls, swatches |
| `r-2` | 3 | Buttons, inputs, rows |
| `r-3` | 5 | Panels |
| `r-4` | 8 | Reserved (focus overlays) |

### Shadows

- Panel: `0 1px 0 oklch(1 0 0 / 0.04) inset, 0 8px 24px oklch(0 0 0 / 0.4)`
- Pop / floating: `0 1px 0 oklch(1 0 0 / 0.04) inset, 0 16px 48px oklch(0 0 0 / 0.55)`

### Status dots

6 px circles with optional 6 px glow. Used in: protocol pills, fixture rows (live state), I/O protocol strip, status bar.

| State | Color | Glow |
|---|---|---|
| live (rx) | `rx` | `0 0 6px oklch(0.80 0.13 150 / 0.6)` |
| tx | `accent` | `0 0 6px oklch(0.78 0.13 215 / 0.5)` |
| warn | `warning` | none |
| idle | `idle` | none |
| error | `error` | none |

---

## Top bar (36 px)

Left to right:

1. **Wordmark** "stage**LX**" — "LX" in `accent`. Followed by a 9 px mono version chip `0.1.0` in a 1 px `border-soft` outline. 14 px right padding, then a 1 px `border-soft` divider.
2. **Show name + state** — "Show" label (11 px muted) · show name (12 px) · live dot · `SAVED 12s ago` (9 px mono muted). Right divider.
3. **Mode tabs** — `Setup` `Patch` `Program` `Run`. Active mode has `bg-panel` fill and `border` outline (3 px radius). Heights 26 px, padding `0 12px`.
4. **Spacer (flex)**.
5. **Protocol pills** — Art-Net / sACN / USB / MIDI / OSC, each a `pill` (18 px tall, 9 px radius) showing live/warn/idle dot + label. Live pills use the green palette; idle pills are gray-only.
6. **FPS / CPU** — mono 10 px metrics.
7. **Settings gear** — 12 px icon button, `fg-muted` → `fg` on hover.

---

## Status bar (22 px)

Mono 10 px throughout. Items separated by middle dot (`·`). Left side: selection count, patched total, universe usage. Right side: venue file, transport state, BPM. The "record armed" indicator uses `rx` color with a `●`.

---

## Panel chrome

Every panel has the same shell. 28 px titlebar (`bg-chrome`, 1 px bottom `border`) containing:

- **Title** (11 px / 600 / 0.04em) — left-aligned.
- **Subtitle** (10 px mono muted) — optional, e.g. `3 fixtures · selected`.
- **Action buttons** — small ghost buttons specific to the panel.
- **Detach** icon (corners-out glyph) — pops the panel out as a floating `egui::Window`.
- **Minimize** icon (single bar) — collapses the body, leaves titlebar visible.

Body padding: 10 px (default), 8 px (`tight`).

---

## Programmer panel

**Width 360 px.** Lives in the left rail. The single most important panel in the app.

### Selection bar (top, 12 px below titlebar)

- 28 px tall, `bg-input` background, 1 px `border-soft`, radius 3.
- Layout: `[tx dot] [mono "1·2·5"] [name "Mac Aura · Sharpy · Sharpy"] [spacer] [mono "3 / 24"]`
- Establishes the "what am I editing?" answer at all times.

### Section: Intensity

- Eyebrow `INTENSITY` + hint `0–100%`.
- **Two vertical faders** side by side, centered, 24 px gap:
  - `Dimmer` — accent fill, 0–100%.
  - `Strobe` — warning (amber) fill, 0 Hz–25 Hz. Below 0.01 reads "OFF".
- Fader spec: 28 px wide, 130 px tall track, `bg-input` with `border`. Tick marks at 0/25/50/75/100. Fill is a vertical gradient `accent → oklch(0.55 0.10 215)` at 0.8 opacity. The cap is a 14 px tall pill with a `border-strong` outline and a 1 px accent stripe across its middle.
- Above each fader: numeric readout (14 px mono, "%" or "Hz" suffix in 10 px muted).
- Below each fader: 10 px label (DIMMER, STROBE).

### Section: Position

- Eyebrow `POSITION` + hint `±270° / ±135°`.
- **Three encoder dials** in a row, 14 px gap: `Pan` (±270°, 1 decimal, sub label `ABS`), `Tilt` (±135°, 1 decimal, `ABS`), `Zoom` (5–45°, 0 decimals, sub label `BEAM`).
- Encoder spec:
  - Outer SVG 76 px, arc radius `size/2 - 6`, arc spans `-135° → +135°`.
  - Track: 2 px stroke `border`, round cap.
  - Fill: 2 px stroke `accent`, round cap, painted from `-135°` to current angle.
  - Indicator dot: 2.5 px radius `accent` at the head of the fill.
  - Hub: inner circle radius `r - 7`, fill `bg-input`, 1 px `border` stroke.
  - Center: 18 px mono value with optional 10 px muted unit (e.g., `°`); below it a 9 px mono sub label.
  - Below dial: 10 px / 500 / 0.06em UPPER label.

### Section: Colour

- Eyebrow `COLOUR` + hint `RGB · 8-bit` + right-aligned mono "255·255·255".
- **Active color row** — `bg-input` strip with: 22 px swatch (1 px black-alpha border), name + small hex/CCT line, `PICK` ghost button.
- **Swatch grid** — 8 columns: White, Red, Amber, Green, Cyan, Blue, Magenta, UV.
  - Each swatch: 28 × 18 px chip with 1 px black-alpha border + 9 px label below.
  - Selected swatch gets a `bg-raised` tile background, `accent-dim` outline, and a 1 px `accent` ring on the chip itself.

### Section: Gobo

- Eyebrow `GOBO` + hint `wheel 1 · 4 slots`.
- **2×2 grid of square tiles**: Open, Dots, Breakup, Star.
  - Aspect 1:1, `bg-input` background, `border-soft` outline.
  - 22 × 22 SVG glyph (see `programmer.jsx` for the four glyphs) + 10 px label.
  - Selected: `bg-raised` background, `accent-dim` outline, glyph stroke becomes `accent`.
- **Spin slider** — horizontal bar with center detent. -3 r/s to +3 r/s. Fill grows from center toward current value. Below 0.05 absolute reads "OFF". Right side mono readout `+1.2 r/s`.

### Quick actions

Footer — 4-up grid of 24 px buttons separated from the rest by a 1 px `border-soft` top border:
- `Black` · `Full` · `Home` · `Reset` (last is ghost).

### Hotkey hint

9 px mono `fg-faint`, centered, multi-segment: `←↑→↓ pan/tilt · +/− dimmer · Z zoom · W/X/C colour`.

### Behavior

- Slider/encoder values bind to the existing `Programmer` resource fields (`dimmer`, `pan`, `tilt`, `zoom`, `strobe`, `color`, `gobo_index`, `gobo_spin`).
- Pan/Tilt math preserved from current code: `pan_deg = (pan - 0.5) * pan_range`. Encoders display the **derived degrees**, not the 0–1 normalized value.
- Strobe rate: `0.0` → "OFF"; otherwise `strobe * 25.0` Hz.
- Zoom angle: `5.0 + zoom * 40.0`° (kept identical to current code).
- Quick actions: `Black` = `dimmer = 0`; `Full` = `dimmer = 1, color = white`; `Home` = `pan = tilt = 0.5`; `Reset` = `*prog = Programmer::default()`.

---

## Patch panel

**Width 580 px** standalone; in the docked layout it sits in the bottom-center region at ~720 px wide.

### Toolbar (above list)

Single row, 6 px gap:
- Filter input — flex 1, 24 px tall, with a 12 px search icon at left padding 7. Placeholder: `Filter by name, type, address…`.
- Quick chips (small buttons): `All` (filled), `Live` (ghost), `U1` (ghost), `U2` (ghost). Active chip uses regular button surface; others ghost.

### Header row

6-column grid, fixed widths: `32 / 1fr / 1.5fr / 0.9fr / 78 / 32`.
Columns: `# · Name · Fixture Type · Mode · Address · (status)`.
Header style: 9 px / 600 / 0.1em UPPER `fg-muted`. Bottom 1 px `border-soft`.

### Rows (24 px tall, dense)

- Fixture index: 3-digit zero-padded mono (`001`), right-aligned, `fg-muted` (or `accent` when selected).
- Name: live/idle dot + 11 px name. Truncates with ellipsis.
- Type: 11 px `fg-secondary`. Truncates.
- Mode: 10 px mono `fg-muted`.
- Address: 11 px mono in `U.CCC` format (`1.001`, `1.017`, `2.078`). Right-aligned. Becomes `fg` when selected.
- Status: 9 px mono `OK` or `—`.
- **Selection visual:** background `oklch(0.24 0.04 215)`, 2 px `accent` left border (replaces alternating stripe).
- **Alternating stripes** on unselected rows: even rows transparent, odd `oklch(0.165 0.005 240 / 0.5)`.
- 1 px bottom border `oklch(0.18 0.005 240)`.

### Footer

Single row, 10 px text:
- Left: `<accent>N</accent> selected · <fg>N</fg> patched · <rx>N</rx> live`
- Right: mono `fg-faint`: `U1 81/512  ·  U2 65/512` (universe channel usage)

### Add Fixture form (below list, in `bg-chrome` card)

10 px padding card, 1 px `border-soft`, 3 px radius. Layout:
- Eyebrow header `ADD FIXTURE` + right-side mono hint `NEXT FREE: 2.078`.
- 6-column grid `1.4fr / 1fr / 1fr / 80 / 80 / auto`, gap 6 px:
  - Type: combo button (Martin · MAC Aura XB) with chevron.
  - Mode: combo button (16ch Extended) with chevron.
  - Name: text input with placeholder `Fixture 12`.
  - Univ: numeric input (mono, right-aligned).
  - Channel: numeric input.
  - **Patch** primary button with plus icon.

### Behavior

- Map to `PatchRes`, `PatchEditState`, `FixtureLibraryRes` from the existing code.
- Click selects (replaces selection); Cmd/Ctrl-click toggles; Shift-click range-extends.
- `Patch` button calls the same `add_fixture` logic; show inline errors using the `error` color in a 2 px-padded line below the form (red text on `bg-input`).
- Validation rules from `patch.rs` are preserved: universe ≥ 1, channel 1–512, type required.
- "NEXT FREE" computes the lowest unallocated address considering each fixture's mode footprint.

---

## Library panel

**Width 420 px.** Three sub-tabs — Fixtures (default) / MVR / Venue. Tab strip uses 1 px `accent` bottom border on active tab; counts in mono shown alongside labels.

### Fixtures tab

- Search row — single filter input with magnifier icon.
- 4-column grid (Manufacturer / Model / Modes / Used). Header in `bg-chrome` with `fg-muted` 9 px UPPER labels. Body in `bg-input`.
- Modes column shows count + first-mode channel count: `4 · 16ch`.
- Used column: `accent` mono number (or `—` in `fg-faint`) showing how many patched fixtures reference this type.
- Below list: **dropzone** — 14 × 12 px padding, 1 px dashed `border-strong`, `bg-input` (60% opacity). Folder icon tile + label/hint + `Browse` button. Accepts `.gdtf` drop.

### MVR tab

- Loaded asset card (12 px padding, `bg-input`, `border-soft`):
  - First row: live dot + filename (12 px / 600) + `Re-import` ghost button.
  - 2-column key/value grid (10 px mono): Embedded GDTFs · Fixtures imported · Path.
- Below: dropzone "Import MVR".

### Venue tab

- Loaded venue card with same pattern: tx dot + filename + `Reload`. Key/value: Format · Tris · Bounds.
- Below: dropzone "Replace Venue" — accepts `.obj`, `.glb`, `.gltf`.

### Behavior

- Wire dropzones to file pickers via `rfd` crate (recommended) — replace the path-string text input from the original `library.rs`. Keep typed-path support: paste a path → enter to load.
- "Used" count comes from `PatchRes` — count fixtures whose `fixture_type_id == ft.fixture_type_id`.
- Errors render as a 1-line band inside the body, `error` color, mono.

---

## DMX I/O panel

**Width 360 px.** Right rail.

### Protocol strip (top)

5-column grid inside a `bg-input` container with 4 px padding. Each cell is a 6 px tall button column:
- `[status dot]` (live/warn/idle)
- 10 px label

Active protocol: `bg-raised` fill + `accent-dim` outline + `fg` text. Inactive: transparent + `fg-secondary`. **Click to switch.**

This single strip replaces the five colored section headers and their stacked configurations from the original `io_panel.rs`. Only the active protocol's config renders below — body height ≈30% of the original.

### Config rows (per protocol)

Two-column grid `76 / 1fr` with 8 px gap. Left column: 10 px / 500 / 0.04em UPPER label (`Bind`, `Dest`, `Universe`, `Receive`, `Mode`, `Priority`, `Port`, `State`). Right: input or toggle.

### Toggle component

Pill switch — 16 × 8 px track + 6 px thumb, smoothly transitions left/right. On state: `accent`/accent thumb (white). Off: `border` track / `fg-muted` thumb. Outer button: `bg-input` (off) or accent-tinted (on), 1 px outline. Includes uppercase 10 px label inside the pill.

### Status banners (inline alerts)

- **Live banner** — `bg-input` background, live dot + 10 px mono message: `bound 0.0.0.0:6454 · 2 nodes seen`.
- **Warning banner** — `oklch(0.22 0.05 80)` bg, `warning-dim` border, warn dot + warning-color mono message: `port busy — close other apps using this device`.

### TX/RX counters (footer)

`bg-chrome` card. 2-column grid:
- Each side: small status dot + 9 px UPPER label (TX / RX), 16 px mono value, 9 px `fg-faint` "packets/s" sub-label.

### Per-protocol contents

| Protocol | Rows | Notes |
|---|---|---|
| **Art-Net** | Bind / Dest / Universe (0–32767) / Receive (toggle + allow-list input) | Live banner shows bind status |
| **sACN** | Mode (TX/RX toggles) / Universe (1–63999) / Priority / Dest | Multicast 239.255.X.X hint |
| **USB DMX** | State / Port / Universe | Combo on Port for serial enumeration; warning banner when busy |
| **MIDI** | State / Port / 8-row CC mapping (Dimmer/Pan/Tilt/Zoom/Red/Green/Blue/Strobe) | "Learn" ghost button on the CC section header |
| **OSC** | State / Port (1024–65535) | Address pattern card showing `/fixture/{id}/{attr}` in accent mono |

### Behavior

- All fields bind to the existing `IoConfig` resource (no field renames needed).
- Counters animate via `IoConfig::*_count` already in the code.
- The "select active protocol" state is local UI state — does not need to persist.

---

## Viewport region

Replaces `draw_viewport_separators` in `lib.rs`.

- Use real CSS-grid-equivalent egui layout (`columns` / `vertical_layout`) — drop the manual `painter.line_segment` calls.
- 1 px `border` between viewports (not 1.5 px gray-70).
- Per-viewport label in the top-left corner: 9 px / 600 / 0.18em UPPER, `accent` color, with optional sub label (e.g., `35mm · persp`, `ortho`) in muted mono.
- **Per-viewport toolbar** (top-right): 3-tab segmented control `Wire / Solid / Beam`. 22 px tall, `bg-chrome` (85% opacity) with `backdrop-filter: blur(8px)` (in egui: a translucent rect — Bevy's egui doesn't support real blur, fall back to flat 92% opacity).
- FOH viewport gets a bottom-right hint: `SHIFT-drag orbit · scroll zoom`.

---

## Reusable atoms

Implement these once as egui widgets:

1. **`Encoder`** — `(label, value, min, max, decimals, unit, sub) -> Response`. Custom-painted with `Painter::circle`, `Painter::add(PathShape)` for the arcs. Click+drag rotates; double-click resets to default.
2. **`Fader`** — `(label, value, range, accent_color) -> Response`. Vertical drag updates value; tick marks painted at 0/25/50/75/100% positions.
3. **`Swatch`** — color tile + label, with selected state.
4. **`StatusDot`** — 6 px circle with optional 6 px shadow ring.
5. **`Pill`** — capsule with optional dot + monospace label.
6. **`Toggle`** — pill switch with sliding thumb.
7. **`Tab`** — generic tab button with optional badge count, supports underline (Library) and bottom-border-with-fill (mode tabs) variants.
8. **`PanelTitlebar`** — 28 px header with title / subtitle / actions / detach / minimize.
9. **`Banner`** — inline status row (live / warning / error), dot + mono message.
10. **`Eyebrow`** — small caps section header (9 px / 600 / 0.14em UPPER mono `fg-muted`).
11. **`Dropzone`** — dashed-border import target with icon, label, hint, and Browse button.

---

## State management

Keep the existing Bevy resources verbatim — the redesign is a presentation-layer change. Specifically:

- `Programmer` (in `stagelx-state`) — all fields used as-is.
- `PatchRes`, `PatchEditState` — used as-is. Add a UI-only `selected_ids: HashSet<FixtureId>` (probably a new `PatchSelection` resource) for the new multi-select model. Existing `add_fixture` validation logic is reused.
- `FixtureLibraryRes`, `VenueLoadState` — unchanged. The "Used" count is derived per-frame from `PatchRes`.
- `IoConfig` — unchanged. Add a UI-only local `active_protocol: enum` state in the I/O panel system.
- Events `SpawnFixtureEvent` / `DespawnFixtureEvent` continue to drive the render layer.

The Plugin signature stays:
```rust
impl Plugin for StageLxUiPlugin { /* … */ }
```

Add new resources (selection, active protocol) inside `build()` via `init_resource`.

---

## Interactions & behavior

### Selection

- Clicking a fixture in the Patch list selects it (replaces). Cmd/Ctrl-click toggles. Shift-click extends range.
- Selection drives the Programmer's "Selection bar" content.
- Programmer changes apply to the current selection.
- ESC clears selection.

### Detach / minimize

- Detach button on a panel titlebar moves that panel from the dock to a floating `egui::Window` (the original code path). Minimize collapses the panel body, leaving the titlebar visible.
- Persist detach/minimize state in a new `UiLayoutState` resource (serializable).

### Hover / focus / pressed

- Buttons: `bg-raised` → `bg-hover` (hover) → `bg-input` (active).
- Inputs: `border-soft` → `border` (hover) → `accent-dim` border + `oklch(0.155 0.005 240)` background (focus).
- Focus visible ring: 1 px `accent` outline at 1 px offset.

### Animations

- Fader cap and encoder indicator: no animation — they track the bound value 1:1 (the value itself can be smoothed at the data layer if desired).
- Toggle thumb: 150 ms ease-out for the left/right slide.
- Tab switching: instant (no fade).

### Errors

- Inline error rows under the offending form, `error` color (red), 11 px mono. Persist until the user changes the input.
- The dropzones surface load errors as a banner in the same place as success messages, swapped color.

---

## Things to avoid (regression guards)

- **Don't bring back per-section saturated headers.** The original used cyan/green/yellow/orange/blue/etc. for each section title — that visual vocabulary is gone. Section headers are uniformly muted.
- **Don't pile floating windows by default.** Default state is the docked layout; floating is opt-in via Detach.
- **Don't reintroduce raw `TX 0  RX 0` strings** for I/O counters — use the two-up counter card.
- **Don't overlay 1.5 px hand-painted lines** between viewports — use real layout borders.
- **Don't expand all 5 protocols at once** in I/O — only one config visible.

---

## Open questions for the implementer

1. Should the docked layout be hard-coded or should panels be freely resizable along their rails? (Recommend: fixed widths in v1, resizable in v2.)
2. Where does the "mode" (Setup / Patch / Program / Run) actually switch behavior? Currently it's just visual — define what each mode locks/unlocks.
3. The Programmer's Pan/Tilt encoders show absolute degrees. Should they accept degree input directly (typed), or only via drag?
4. MVR re-import: re-loads from disk, or just re-runs the in-memory parse? Current handoff assumes the former.

Tag the user (`@stageLX-team`) when blocked on any of the above.
