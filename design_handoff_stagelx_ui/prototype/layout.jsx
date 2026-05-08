/* global React, Atoms, ProgrammerPanel, PatchPanel, LibraryPanel, IoPanelView */

const { Icon: TopIcon } = Atoms;

// ── Top bar ──────────────────────────────────────────────────────────
function TopBar() {
  return (
    <div style={{
      height: 36,
      display: "flex",
      alignItems: "center",
      gap: 0,
      padding: "0 10px",
      background: "var(--bg-chrome)",
      borderBottom: "1px solid var(--border)",
      flexShrink: 0,
    }}>
      {/* wordmark */}
      <div style={{ display: "flex", alignItems: "center", gap: 8, paddingRight: 14, borderRight: "1px solid var(--border-soft)", height: "100%" }}>
        <span style={{ fontSize: 14, fontWeight: 600, letterSpacing: "-0.01em", color: "var(--fg)" }}>
          stage<span style={{ color: "var(--accent)" }}>LX</span>
        </span>
        <span className="mono" style={{ fontSize: 9, color: "var(--fg-faint)", padding: "1px 5px", border: "1px solid var(--border-soft)", borderRadius: 2 }}>0.1.0</span>
      </div>

      {/* show name + state */}
      <div style={{ display: "flex", alignItems: "center", gap: 8, padding: "0 14px", height: "100%", borderRight: "1px solid var(--border-soft)" }}>
        <span style={{ fontSize: 11, color: "var(--fg-muted)" }}>Show</span>
        <span style={{ fontSize: 12, color: "var(--fg)", fontWeight: 500 }}>tour-2026-mainstage</span>
        <span className="dot live" />
        <span className="mono" style={{ fontSize: 9, color: "var(--fg-muted)" }}>SAVED 12s ago</span>
      </div>

      {/* mode tabs */}
      <div style={{ display: "flex", gap: 2, padding: "0 8px" }}>
        {["Setup", "Patch", "Program", "Run"].map((m, i) => (
          <button key={m} style={{
            height: 26, padding: "0 12px",
            background: i === 2 ? "var(--bg-panel)" : "transparent",
            border: i === 2 ? "1px solid var(--border)" : "1px solid transparent",
            borderRadius: 3,
            fontSize: 11, fontWeight: i === 2 ? 600 : 500,
            color: i === 2 ? "var(--fg)" : "var(--fg-secondary)",
            letterSpacing: "0.02em",
          }}>{m}</button>
        ))}
      </div>

      <span style={{ flex: 1 }} />

      {/* Status pills */}
      <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
        <span className="pill live"><span className="dot live" />Art-Net</span>
        <span className="pill live"><span className="dot live" />sACN</span>
        <span className="pill"><span className="dot warn" />USB</span>
        <span className="pill idle">MIDI</span>
        <span className="pill live"><span className="dot live" />OSC</span>
      </div>

      <div style={{ width: 14 }} />

      {/* CPU/FPS */}
      <div style={{ display: "flex", alignItems: "center", gap: 10, fontFamily: "var(--font-mono)", fontSize: 10, color: "var(--fg-muted)" }}>
        <span>FPS <span style={{ color: "var(--fg)" }}>59.9</span></span>
        <span>CPU <span style={{ color: "var(--fg)" }}>14%</span></span>
      </div>

      <div style={{ width: 12 }} />

      <button className="btn small ghost icon" title="Settings">
        <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
          <circle cx="8" cy="8" r="2" stroke="currentColor" strokeWidth="1.5"/>
          <path d="M8 1v2M8 13v2M1 8h2M13 8h2M3 3l1.5 1.5M11.5 11.5L13 13M3 13l1.5-1.5M11.5 4.5L13 3" stroke="currentColor" strokeWidth="1.5"/>
        </svg>
      </button>
    </div>
  );
}

// ── Viewport ─────────────────────────────────────────────────────────
function ViewportLabel({ name, sub, top, left, right, bottom }) {
  return (
    <div style={{ position: "absolute", top, left, right, bottom, display: "flex", alignItems: "center", gap: 8, pointerEvents: "none" }}>
      <span className="mono" style={{ fontSize: 9, fontWeight: 600, letterSpacing: "0.18em", color: "var(--accent)" }}>{name}</span>
      {sub && <span className="mono" style={{ fontSize: 9, color: "var(--fg-muted)" }}>{sub}</span>}
    </div>
  );
}

function ViewportRegion() {
  // Stylized stage + grid placeholders
  return (
    <div style={{ position: "relative", flex: 1, background: "var(--bg-app)", display: "grid", gridTemplateColumns: "3fr 1fr", gridTemplateRows: "1fr 1fr", gap: 1, overflow: "hidden" }}>
      {/* FOH (large) */}
      <div style={{ gridColumn: "1", gridRow: "1 / span 2", position: "relative", background: "radial-gradient(ellipse at 50% 70%, oklch(0.18 0.005 240) 0%, oklch(0.13 0.004 240) 80%)", overflow: "hidden" }}>
        <StageRender />
        <ViewportLabel name="FOH" sub="35mm · persp" top={10} left={12} />
        <div style={{ position: "absolute", bottom: 10, right: 12, display: "flex", gap: 4, alignItems: "center" }}>
          <span className="mono" style={{ fontSize: 9, color: "var(--fg-faint)" }}>SHIFT-drag orbit · scroll zoom</span>
        </div>
        <ViewportToolbar />
      </div>

      {/* TOP */}
      <div style={{ gridColumn: "2", gridRow: "1", position: "relative", background: "var(--bg-app)", borderLeft: "1px solid var(--border)", overflow: "hidden" }}>
        <TopView />
        <ViewportLabel name="TOP" sub="ortho" top={10} left={12} />
      </div>

      {/* SIDE */}
      <div style={{ gridColumn: "2", gridRow: "2", position: "relative", background: "var(--bg-app)", borderTop: "1px solid var(--border)", borderLeft: "1px solid var(--border)", overflow: "hidden" }}>
        <SideView />
        <ViewportLabel name="SIDE" sub="ortho" top={10} left={12} />
      </div>
    </div>
  );
}

const ViewportToolbar = () => (
  <div style={{
    position: "absolute", top: 10, right: 12,
    display: "flex", gap: 2, padding: 2,
    background: "oklch(0.165 0.005 240 / 0.85)",
    border: "1px solid var(--border-soft)",
    borderRadius: 3, backdropFilter: "blur(8px)",
  }}>
    {["Wire", "Solid", "Beam"].map((m, i) => (
      <button key={m} style={{
        height: 20, padding: "0 8px",
        background: i === 2 ? "var(--bg-raised)" : "transparent",
        borderRadius: 2,
        fontSize: 10, fontWeight: 500,
        color: i === 2 ? "var(--accent)" : "var(--fg-secondary)",
      }}>{m}</button>
    ))}
  </div>
);

// Stylized stage placeholder for FOH view
const StageRender = () => (
  <svg width="100%" height="100%" viewBox="0 0 600 380" preserveAspectRatio="xMidYMid slice" style={{ display: "block" }}>
    <defs>
      <linearGradient id="floor" x1="0" y1="0" x2="0" y2="1">
        <stop offset="0%" stopColor="oklch(0.20 0.005 240)" />
        <stop offset="100%" stopColor="oklch(0.13 0.004 240)" />
      </linearGradient>
      <radialGradient id="beam" cx="50%" cy="0%" r="80%">
        <stop offset="0%" stopColor="oklch(0.78 0.13 215 / 0.55)" />
        <stop offset="100%" stopColor="oklch(0.78 0.13 215 / 0)" />
      </radialGradient>
      <radialGradient id="beamA" cx="50%" cy="0%" r="80%">
        <stop offset="0%" stopColor="oklch(0.80 0.18 70 / 0.5)" />
        <stop offset="100%" stopColor="oklch(0.80 0.18 70 / 0)" />
      </radialGradient>
      <radialGradient id="beamM" cx="50%" cy="0%" r="80%">
        <stop offset="0%" stopColor="oklch(0.65 0.22 340 / 0.45)" />
        <stop offset="100%" stopColor="oklch(0.65 0.22 340 / 0)" />
      </radialGradient>
    </defs>
    {/* floor grid */}
    <g stroke="oklch(0.28 0.005 240)" strokeWidth="0.5" fill="none">
      {Array.from({ length: 14 }).map((_, i) => {
        const y = 220 + i * 12;
        const inset = i * 8;
        return <line key={`h${i}`} x1={20 + inset} y1={y} x2={580 - inset} y2={y} opacity={0.6 - i * 0.03} />;
      })}
      {Array.from({ length: 12 }).map((_, i) => {
        const x = 50 + i * 50;
        return <line key={`v${i}`} x1={x} y1="220" x2={300 + (x - 300) * 0.4} y2="380" opacity="0.4" />;
      })}
    </g>
    <rect x="0" y="220" width="600" height="160" fill="url(#floor)" opacity="0.5" />

    {/* truss */}
    <g stroke="oklch(0.40 0.005 240)" strokeWidth="1.5" fill="oklch(0.22 0.005 240)">
      <rect x="80" y="60" width="440" height="8" rx="1" />
      <rect x="80" y="100" width="440" height="6" rx="1" />
    </g>

    {/* beams */}
    <ellipse cx="180" cy="68" rx="60" ry="180" fill="url(#beam)" />
    <ellipse cx="300" cy="68" rx="50" ry="180" fill="url(#beamA)" />
    <ellipse cx="420" cy="68" rx="60" ry="180" fill="url(#beamM)" />

    {/* fixtures on truss */}
    {[120, 180, 240, 300, 360, 420, 480].map((x, i) => (
      <g key={i}>
        <rect x={x - 6} y="64" width="12" height="14" fill="oklch(0.30 0.005 240)" stroke="oklch(0.45 0.005 240)" strokeWidth="0.5" rx="1" />
        <circle cx={x} cy="80" r="3" fill={i === 1 ? "oklch(0.78 0.13 215)" : i === 3 ? "oklch(0.80 0.18 70)" : i === 5 ? "oklch(0.65 0.22 340)" : "oklch(0.30 0.005 240)"} />
      </g>
    ))}
  </svg>
);

const TopView = () => (
  <svg width="100%" height="100%" viewBox="0 0 200 200" preserveAspectRatio="xMidYMid slice">
    <g stroke="oklch(0.26 0.005 240)" strokeWidth="0.5" fill="none">
      {Array.from({ length: 11 }).map((_, i) => <line key={`h${i}`} x1="10" x2="190" y1={10 + i * 18} y2={10 + i * 18} />)}
      {Array.from({ length: 11 }).map((_, i) => <line key={`v${i}`} y1="10" y2="190" x1={10 + i * 18} x2={10 + i * 18} />)}
    </g>
    <rect x="40" y="40" width="120" height="80" fill="none" stroke="oklch(0.50 0.005 240)" strokeWidth="0.8" strokeDasharray="2 2" />
    {[60, 80, 100, 120, 140].map((x, i) => (
      <circle key={i} cx={x} cy="40" r="2.5" fill={i === 1 ? "oklch(0.78 0.13 215)" : i === 2 ? "oklch(0.80 0.18 70)" : "oklch(0.55 0.005 240)"} />
    ))}
  </svg>
);

const SideView = () => (
  <svg width="100%" height="100%" viewBox="0 0 200 200" preserveAspectRatio="xMidYMid slice">
    <g stroke="oklch(0.26 0.005 240)" strokeWidth="0.5" fill="none">
      {Array.from({ length: 11 }).map((_, i) => <line key={`h${i}`} x1="10" x2="190" y1={10 + i * 18} y2={10 + i * 18} />)}
    </g>
    <line x1="10" y1="160" x2="190" y2="160" stroke="oklch(0.50 0.005 240)" strokeWidth="1" />
    <rect x="40" y="40" width="120" height="6" fill="oklch(0.30 0.005 240)" stroke="oklch(0.45 0.005 240)" />
    {[60, 80, 100, 120, 140].map((x, i) => (
      <g key={i}>
        <line x1={x} y1="46" x2={x + (i - 2) * 6} y2="160" stroke="oklch(0.78 0.13 215 / 0.25)" strokeWidth="1.5" />
        <circle cx={x} cy="46" r="2.5" fill={i === 1 ? "oklch(0.78 0.13 215)" : i === 2 ? "oklch(0.80 0.18 70)" : "oklch(0.55 0.005 240)"} />
      </g>
    ))}
  </svg>
);

// ── Bottom status bar ────────────────────────────────────────────────
function StatusBar() {
  return (
    <div style={{
      height: 22,
      display: "flex", alignItems: "center", gap: 14, padding: "0 12px",
      background: "var(--bg-chrome)", borderTop: "1px solid var(--border)",
      fontFamily: "var(--font-mono)", fontSize: 10, color: "var(--fg-muted)",
      flexShrink: 0,
    }}>
      <span><span style={{ color: "var(--accent)" }}>3</span> selected</span>
      <span>·</span>
      <span><span style={{ color: "var(--fg)" }}>11</span> patched / <span style={{ color: "var(--fg)" }}>2</span> universes</span>
      <span>·</span>
      <span>U1 <span style={{ color: "var(--fg)" }}>81/512</span></span>
      <span>U2 <span style={{ color: "var(--fg)" }}>65/512</span></span>
      <span style={{ flex: 1 }} />
      <span>arena-mainstage.glb</span>
      <span>·</span>
      <span style={{ color: "var(--rx)" }}>● record armed</span>
      <span>·</span>
      <span>BPM <span style={{ color: "var(--fg)" }}>128.0</span></span>
    </div>
  );
}

// ── Full hybrid layout ───────────────────────────────────────────────
function FullLayout() {
  return (
    <div style={{ width: 1440, height: 880, display: "flex", flexDirection: "column", background: "var(--bg-app)", border: "1px solid var(--border)", borderRadius: 4, overflow: "hidden" }}>
      <TopBar />
      <div style={{ flex: 1, display: "grid", gridTemplateColumns: "300px 1fr 320px", minHeight: 0 }}>
        {/* Left rail */}
        <div style={{ display: "flex", flexDirection: "column", borderRight: "1px solid var(--border)", background: "var(--bg-chrome)" }}>
          <RailHeader title="Programmer" detachable />
          <div style={{ padding: 8, flex: 1, overflow: "auto" }}>
            <ProgrammerPanelInline />
          </div>
        </div>

        {/* Center: viewports + bottom panels */}
        <div style={{ display: "flex", flexDirection: "column", minWidth: 0 }}>
          <ViewportRegion />
          <div style={{ height: 248, borderTop: "1px solid var(--border)", display: "grid", gridTemplateColumns: "1.4fr 1fr", background: "var(--bg-chrome)" }}>
            <div style={{ display: "flex", flexDirection: "column", borderRight: "1px solid var(--border)" }}>
              <RailHeader title="Patch" />
              <div style={{ padding: 8, flex: 1, overflow: "auto" }}><PatchPanelInline /></div>
            </div>
            <div style={{ display: "flex", flexDirection: "column" }}>
              <RailHeader title="Library" />
              <div style={{ padding: 8, flex: 1, overflow: "auto" }}><LibraryPanelInline /></div>
            </div>
          </div>
        </div>

        {/* Right rail */}
        <div style={{ display: "flex", flexDirection: "column", borderLeft: "1px solid var(--border)", background: "var(--bg-chrome)" }}>
          <RailHeader title="DMX I/O" />
          <div style={{ padding: 8, flex: 1, overflow: "auto" }}><IoPanelInline /></div>
        </div>
      </div>
      <StatusBar />
    </div>
  );
}

const RailHeader = ({ title, detachable }) => (
  <div style={{
    height: 28, display: "flex", alignItems: "center", gap: 6,
    padding: "0 10px",
    background: "oklch(0.18 0.005 240)",
    borderBottom: "1px solid var(--border)",
  }}>
    <span style={{ fontSize: 10, fontWeight: 600, letterSpacing: "0.14em", textTransform: "uppercase", color: "var(--fg-secondary)" }}>{title}</span>
    <span style={{ flex: 1 }} />
    {detachable && (
      <button className="btn ghost small icon" title="Detach to floating window">
        <Atoms.Icon name="detach" />
      </button>
    )}
  </div>
);

// Inline variants — same content, no panel chrome
function ProgrammerPanelInline() {
  return <div style={{ marginTop: -8 }}><ProgrammerPanel /></div>;
}
function PatchPanelInline() {
  return <div style={{ marginTop: -8 }}><PatchPanel /></div>;
}
function LibraryPanelInline() {
  return <div style={{ marginTop: -8 }}><LibraryPanel /></div>;
}
function IoPanelInline() {
  return <div style={{ marginTop: -8 }}><IoPanelView /></div>;
}

window.FullLayout = FullLayout;
