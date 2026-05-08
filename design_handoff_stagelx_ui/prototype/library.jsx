/* global React, Atoms */
const { useState: useLibState } = React;
const { Panel: LPanel, Icon: LIcon } = Atoms;

const FIXTURE_TYPES = [
  { mfr: "Martin",   name: "MAC Aura XB",        modes: 4, channels: "16 / 23 / 8 / 4",   used: 4 },
  { mfr: "Martin",   name: "Atomic 3000 LED",    modes: 2, channels: "11 / 4",            used: 1 },
  { mfr: "Clay Paky", name: "Sharpy",            modes: 2, channels: "16 / 36",           used: 2 },
  { mfr: "Robe",     name: "Robin Pointe",       modes: 3, channels: "27 / 36 / 24",      used: 2 },
  { mfr: "ETC",      name: "ColorSource PAR",    modes: 4, channels: "5 / 6 / 7 / 8",     used: 2 },
  { mfr: "Chauvet",  name: "Maverick MK2 Spot",  modes: 1, channels: "39",                used: 0 },
  { mfr: "Ayrton",   name: "Mistral-TC",         modes: 2, channels: "26 / 38",           used: 0 },
];

function LibraryPanel() {
  const [tab, setTab] = useLibState("fixtures");
  const [query, setQuery] = useLibState("");

  return (
    <LPanel
      title="Library"
      subtitle={`${FIXTURE_TYPES.length} fixture types`}
      style={{ width: 420 }}
      actions={
        <button className="btn small">
          <LIcon name="folder" /> IMPORT
        </button>
      }
    >
      {/* Sub-tabs */}
      <div style={{ display: "flex", gap: 0, marginBottom: 10, borderBottom: "1px solid var(--border-soft)" }}>
        {[
          ["fixtures", "Fixtures",  FIXTURE_TYPES.length],
          ["mvr",      "MVR Scenes", 1],
          ["venue",    "Venue",      1],
        ].map(([id, label, count]) => (
          <button
            key={id}
            onClick={() => setTab(id)}
            style={{
              height: 26, padding: "0 12px",
              borderBottom: tab === id ? "1px solid var(--accent)" : "1px solid transparent",
              color: tab === id ? "var(--fg)" : "var(--fg-secondary)",
              fontSize: 11, fontWeight: tab === id ? 600 : 500,
              display: "inline-flex", alignItems: "center", gap: 6,
              marginBottom: -1,
            }}
          >
            {label}
            <span className="mono" style={{ fontSize: 9, color: tab === id ? "var(--accent)" : "var(--fg-muted)" }}>{count}</span>
          </button>
        ))}
      </div>

      {tab === "fixtures" && (
        <>
          {/* Search + import */}
          <div style={{ display: "flex", gap: 6, marginBottom: 8 }}>
            <div style={{ position: "relative", flex: 1 }}>
              <span style={{ position: "absolute", left: 7, top: "50%", transform: "translateY(-50%)", color: "var(--fg-muted)", display: "flex" }}>
                <LIcon name="search" />
              </span>
              <input className="input" placeholder="Search manufacturer, model…" value={query} onChange={(e) => setQuery(e.target.value)} style={{ paddingLeft: 24 }} />
            </div>
          </div>

          {/* List */}
          <div style={{
            display: "grid",
            gridTemplateColumns: "1fr 1.4fr 100px 60px",
            gap: 0,
            background: "var(--bg-input)",
            border: "1px solid var(--border-soft)",
            borderRadius: 3,
            overflow: "hidden",
            maxHeight: 220,
            overflowY: "auto",
          }}>
            {/* header */}
            {["Manufacturer", "Model", "Modes", "Used"].map((h, i) => (
              <div key={i} style={{
                fontSize: 9, fontWeight: 600, letterSpacing: "0.1em", textTransform: "uppercase",
                color: "var(--fg-muted)",
                padding: "6px 8px",
                background: "var(--bg-chrome)",
                borderBottom: "1px solid var(--border-soft)",
                textAlign: i >= 2 ? "right" : "left",
              }}>{h}</div>
            ))}
            {FIXTURE_TYPES.map((ft, i) => (
              <React.Fragment key={i}>
                <div style={{ padding: "6px 8px", fontSize: 11, color: "var(--fg-secondary)", borderTop: i ? "1px solid oklch(0.18 0.005 240)" : "none" }}>{ft.mfr}</div>
                <div style={{ padding: "6px 8px", fontSize: 11, color: "var(--fg)", borderTop: i ? "1px solid oklch(0.18 0.005 240)" : "none" }}>{ft.name}</div>
                <div className="mono" style={{ padding: "6px 8px", fontSize: 10, color: "var(--fg-muted)", borderTop: i ? "1px solid oklch(0.18 0.005 240)" : "none", textAlign: "right" }}>{ft.modes} · {ft.channels.split(" / ")[0]}ch</div>
                <div className="mono" style={{ padding: "6px 8px", fontSize: 11, color: ft.used ? "var(--accent)" : "var(--fg-faint)", borderTop: i ? "1px solid oklch(0.18 0.005 240)" : "none", textAlign: "right" }}>
                  {ft.used || "—"}
                </div>
              </React.Fragment>
            ))}
          </div>

          {/* Drop zone */}
          <DropZone
            label="Import GDTF"
            hint=".gdtf · drag from finder or click to browse"
            icon="folder"
          />
        </>
      )}

      {tab === "mvr" && (
        <>
          <div style={{ padding: 12, background: "var(--bg-input)", border: "1px solid var(--border-soft)", borderRadius: 3, marginBottom: 8 }}>
            <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 6 }}>
              <span className="dot live" />
              <span style={{ fontSize: 12, color: "var(--fg)", fontWeight: 600 }}>Tour 2026 — Main Stage.mvr</span>
              <span style={{ flex: 1 }} />
              <button className="btn ghost small">Re-import</button>
            </div>
            <div style={{ display: "grid", gridTemplateColumns: "auto 1fr", gap: "2px 12px", fontSize: 10, fontFamily: "var(--font-mono)" }}>
              <span style={{ color: "var(--fg-muted)" }}>Embedded GDTFs</span><span style={{ color: "var(--fg)" }}>7</span>
              <span style={{ color: "var(--fg-muted)" }}>Fixtures imported</span><span style={{ color: "var(--fg)" }}>11</span>
              <span style={{ color: "var(--fg-muted)" }}>Path</span><span style={{ color: "var(--fg-secondary)", overflow: "hidden", textOverflow: "ellipsis" }}>~/Documents/MVR/tour-2026.mvr</span>
            </div>
          </div>
          <DropZone label="Import MVR" hint="loads embedded GDTFs and populates patch" icon="folder" />
        </>
      )}

      {tab === "venue" && (
        <>
          <div style={{ padding: 12, background: "var(--bg-input)", border: "1px solid var(--border-soft)", borderRadius: 3, marginBottom: 8 }}>
            <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 6 }}>
              <span className="dot tx" />
              <span style={{ fontSize: 12, color: "var(--fg)", fontWeight: 600 }}>arena-mainstage.glb</span>
              <span style={{ flex: 1 }} />
              <button className="btn ghost small">Reload</button>
            </div>
            <div style={{ display: "grid", gridTemplateColumns: "auto 1fr", gap: "2px 12px", fontSize: 10, fontFamily: "var(--font-mono)" }}>
              <span style={{ color: "var(--fg-muted)" }}>Format</span><span style={{ color: "var(--fg)" }}>glTF Binary</span>
              <span style={{ color: "var(--fg-muted)" }}>Tris</span><span style={{ color: "var(--fg)" }}>184,302</span>
              <span style={{ color: "var(--fg-muted)" }}>Bounds</span><span style={{ color: "var(--fg)" }}>32.4 × 18.6 × 12.0 m</span>
            </div>
          </div>
          <DropZone label="Replace Venue" hint="OBJ · GLB · glTF — replaces the loaded venue" icon="folder" />
        </>
      )}
    </LPanel>
  );
}

const DropZone = ({ label, hint, icon }) => (
  <div style={{
    marginTop: 10,
    padding: "14px 12px",
    border: "1px dashed var(--border-strong)",
    borderRadius: 3,
    background: "oklch(0.155 0.004 240 / 0.6)",
    display: "flex", alignItems: "center", gap: 10,
  }}>
    <div style={{
      width: 28, height: 28, borderRadius: 3,
      background: "var(--bg-raised)", border: "1px solid var(--border)",
      display: "grid", placeItems: "center", color: "var(--fg-secondary)",
    }}>
      <Atoms.Icon name={icon} size={14} />
    </div>
    <div style={{ display: "flex", flexDirection: "column", flex: 1, gap: 2 }}>
      <span style={{ fontSize: 11, color: "var(--fg)", fontWeight: 500 }}>{label}</span>
      <span className="mono" style={{ fontSize: 10, color: "var(--fg-muted)" }}>{hint}</span>
    </div>
    <button className="btn small">Browse</button>
  </div>
);

window.LibraryPanel = LibraryPanel;
