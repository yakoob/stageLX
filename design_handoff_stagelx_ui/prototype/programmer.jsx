/* global React, Atoms */
const { useState: useStateP } = React;
const { Panel: PPanel, Section: PSection, Encoder, Fader, Swatch, Icon: PIcon } = Atoms;

function ProgrammerPanel() {
  const [color, setColor] = useStateP("White");
  const [gobo, setGobo] = useStateP("Open");
  const [dimmer, setDimmer] = useStateP(74);
  const [pan, setPan] = useStateP(-12.4);
  const [tilt, setTilt] = useStateP(28.7);
  const [zoom, setZoom] = useStateP(18);
  const [strobe, setStrobe] = useStateP(0);

  const colors = [
    ["White",   "oklch(0.97 0.01 90)"],
    ["Red",     "oklch(0.62 0.22 28)"],
    ["Amber",   "oklch(0.78 0.16 70)"],
    ["Green",   "oklch(0.70 0.20 145)"],
    ["Cyan",    "oklch(0.78 0.13 200)"],
    ["Blue",    "oklch(0.55 0.20 260)"],
    ["Magenta", "oklch(0.65 0.23 340)"],
    ["UV",      "oklch(0.40 0.20 295)"],
  ];

  const gobos = ["Open", "Dots", "Breakup", "Star"];

  return (
    <PPanel
      title="Programmer"
      subtitle="3 fixtures · selected"
      style={{ width: 360 }}
      actions={
        <button className="btn ghost small">
          <PIcon name="lock" /> CLEAR
        </button>
      }
    >
      {/* Selection bar */}
      <div style={{
        display: "flex", alignItems: "center", gap: 6,
        padding: "6px 8px", marginBottom: 12,
        background: "var(--bg-input)", border: "1px solid var(--border-soft)", borderRadius: 3,
      }}>
        <span className="dot tx" />
        <span className="mono" style={{ fontSize: 11, color: "var(--fg)" }}>1·2·5</span>
        <span style={{ fontSize: 11, color: "var(--fg-muted)" }}>Mac Aura · Sharpy · Sharpy</span>
        <span style={{ flex: 1 }} />
        <span className="mono" style={{ fontSize: 10, color: "var(--fg-muted)" }}>3 / 24</span>
      </div>

      {/* Dimmer + Strobe (faders) */}
      <PSection label="Intensity" hint="0–100%">
        <div style={{ display: "flex", gap: 24, justifyContent: "center", padding: "4px 0" }}>
          <Fader label="Dimmer" value={dimmer} unit="%" />
          <Fader label="Strobe" value={strobe} unit="Hz" accent="var(--warning)" />
        </div>
      </PSection>

      {/* Position encoders */}
      <PSection label="Position" hint="±270° / ±135°">
        <div style={{ display: "flex", gap: 14, justifyContent: "center" }}>
          <Encoder label="Pan"  value={pan}  unit="°" decimals={1} min={-270} max={270} sub="ABS" />
          <Encoder label="Tilt" value={tilt} unit="°" decimals={1} min={-135} max={135} sub="ABS" />
          <Encoder label="Zoom" value={zoom} unit="°" decimals={0} min={5}    max={45}  sub="BEAM" />
        </div>
      </PSection>

      {/* Colour */}
      <PSection
        label="Colour"
        hint="RGB · 8-bit"
        action={
          <span className="mono" style={{ fontSize: 10, color: "var(--fg-muted)" }}>
            255·255·255
          </span>
        }
      >
        <div style={{
          display: "flex", alignItems: "center", gap: 8,
          padding: "6px 8px",
          background: "var(--bg-input)", border: "1px solid var(--border-soft)", borderRadius: 3,
          marginBottom: 8,
        }}>
          <div style={{ width: 22, height: 22, borderRadius: 2, background: colors.find(c => c[0] === color)?.[1] || "white", border: "1px solid oklch(0 0 0 / 0.4)" }} />
          <div style={{ display: "flex", flexDirection: "column", flex: 1, gap: 1 }}>
            <span style={{ fontSize: 11, color: "var(--fg)" }}>{color}</span>
            <span className="mono" style={{ fontSize: 9, color: "var(--fg-muted)" }}>#FFFFFF · 6500K</span>
          </div>
          <button className="btn ghost small">PICK</button>
        </div>
        <div style={{ display: "grid", gridTemplateColumns: "repeat(8, 1fr)", gap: 2 }}>
          {colors.map(([name, c]) => (
            <Swatch key={name} color={c} label={name} selected={color === name} onClick={() => setColor(name)} />
          ))}
        </div>
      </PSection>

      {/* Gobo */}
      <PSection label="Gobo" hint="wheel 1 · 4 slots">
        <div style={{ display: "grid", gridTemplateColumns: "repeat(4, 1fr)", gap: 4 }}>
          {gobos.map((name) => (
            <button
              key={name}
              onClick={() => setGobo(name)}
              style={{
                aspectRatio: "1",
                background: gobo === name ? "var(--bg-raised)" : "var(--bg-input)",
                border: `1px solid ${gobo === name ? "var(--accent-dim)" : "var(--border-soft)"}`,
                borderRadius: 3,
                display: "flex", flexDirection: "column", alignItems: "center", justifyContent: "center", gap: 4,
                color: gobo === name ? "var(--fg)" : "var(--fg-secondary)",
              }}
            >
              <GoboIcon kind={name} active={gobo === name} />
              <span style={{ fontSize: 10, letterSpacing: "0.02em" }}>{name}</span>
            </button>
          ))}
        </div>
        <div style={{ display: "flex", alignItems: "center", gap: 8, marginTop: 8 }}>
          <span style={{ fontSize: 10, color: "var(--fg-muted)", letterSpacing: "0.06em", textTransform: "uppercase" }}>Spin</span>
          <div style={{ flex: 1, height: 4, background: "var(--bg-input)", border: "1px solid var(--border-soft)", borderRadius: 2, position: "relative" }}>
            <div style={{ position: "absolute", left: "50%", top: -1, bottom: -1, width: 1, background: "var(--border-strong)" }} />
            <div style={{ position: "absolute", left: "50%", top: 0, bottom: 0, width: "20%", background: "var(--accent)", borderRadius: 2 }} />
          </div>
          <span className="mono" style={{ fontSize: 11, color: "var(--fg)", minWidth: 56, textAlign: "right" }}>+1.2 r/s</span>
        </div>
      </PSection>

      {/* Quick actions */}
      <div style={{ display: "grid", gridTemplateColumns: "repeat(4, 1fr)", gap: 4, paddingTop: 8, borderTop: "1px solid var(--border-soft)" }}>
        <button className="btn small">Black</button>
        <button className="btn small">Full</button>
        <button className="btn small">Home</button>
        <button className="btn small ghost">Reset</button>
      </div>

      {/* Hotkey hint */}
      <div style={{ marginTop: 8, padding: "5px 0", fontSize: 9, color: "var(--fg-faint)", letterSpacing: "0.02em", fontFamily: "var(--font-mono)", textAlign: "center" }}>
        ←↑→↓ pan/tilt &nbsp;·&nbsp; +/− dimmer &nbsp;·&nbsp; Z zoom &nbsp;·&nbsp; W/X/C colour
      </div>
    </PPanel>
  );
}

// Tiny gobo glyphs
const GoboIcon = ({ kind, active }) => {
  const c = active ? "var(--accent)" : "var(--fg-muted)";
  const bg = "var(--bg-app)";
  switch (kind) {
    case "Open":
      return <svg width="22" height="22" viewBox="0 0 22 22"><circle cx="11" cy="11" r="8" fill={bg} stroke={c} strokeWidth="1" /></svg>;
    case "Dots":
      return (
        <svg width="22" height="22" viewBox="0 0 22 22">
          <circle cx="11" cy="11" r="8" fill={bg} stroke={c} strokeWidth="1" />
          {[[8,8],[14,8],[8,14],[14,14],[11,11]].map(([x,y],i) => <circle key={i} cx={x} cy={y} r="1.2" fill={c} />)}
        </svg>
      );
    case "Breakup":
      return (
        <svg width="22" height="22" viewBox="0 0 22 22">
          <circle cx="11" cy="11" r="8" fill={bg} stroke={c} strokeWidth="1" />
          <path d="M6 9l3-1 3 2 4-1M5 14l4 1 3-2 4 0" stroke={c} strokeWidth="0.8" fill="none" />
        </svg>
      );
    case "Star":
      return (
        <svg width="22" height="22" viewBox="0 0 22 22">
          <circle cx="11" cy="11" r="8" fill={bg} stroke={c} strokeWidth="1" />
          <path d="M11 6l1.4 3.2L16 10l-3 2 1 3.5L11 14l-3 1.5L9 12l-3-2 3.6-0.8z" fill={c} />
        </svg>
      );
    default: return null;
  }
};

window.ProgrammerPanel = ProgrammerPanel;
