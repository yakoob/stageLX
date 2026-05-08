/* global React, Atoms */
const { useState: usePatchState } = React;
const { Panel: PaPanel, Section: PaSection, Icon: PaIcon } = Atoms;

const FIXTURES = [
  { id: 1,  name: "MAC Aura SL · 01",   type: "Martin · MAC Aura XB",     mode: "16ch Extended", u: 1, ch: 1,   live: true },
  { id: 2,  name: "MAC Aura SL · 02",   type: "Martin · MAC Aura XB",     mode: "16ch Extended", u: 1, ch: 17,  live: true },
  { id: 3,  name: "MAC Aura SL · 03",   type: "Martin · MAC Aura XB",     mode: "16ch Extended", u: 1, ch: 33,  live: true },
  { id: 4,  name: "MAC Aura SL · 04",   type: "Martin · MAC Aura XB",     mode: "16ch Extended", u: 1, ch: 49,  live: false },
  { id: 5,  name: "Sharpy · SR",        type: "Clay Paky · Sharpy",       mode: "16ch Standard", u: 1, ch: 65,  live: true },
  { id: 6,  name: "Sharpy · SL",        type: "Clay Paky · Sharpy",       mode: "16ch Standard", u: 1, ch: 81,  live: true },
  { id: 7,  name: "Robin Pointe · 01",  type: "Robe · Robin Pointe",      mode: "Mode 2",        u: 2, ch: 1,   live: false },
  { id: 8,  name: "Robin Pointe · 02",  type: "Robe · Robin Pointe",      mode: "Mode 2",        u: 2, ch: 28,  live: false },
  { id: 9,  name: "ColorSource PAR",    type: "ETC · ColorSource PAR",    mode: "RGB-IA · 5ch",  u: 2, ch: 55,  live: true },
  { id: 10, name: "ColorSource PAR",    type: "ETC · ColorSource PAR",    mode: "RGB-IA · 5ch",  u: 2, ch: 60,  live: true },
  { id: 11, name: "Atomic 3000",        type: "Martin · Atomic 3000",     mode: "DMX",           u: 2, ch: 65,  live: true },
];

function PatchPanel() {
  const [sel, setSel] = usePatchState(new Set([1, 2, 5]));
  const [filter, setFilter] = usePatchState("");

  const toggle = (id, e) => {
    const next = new Set(sel);
    if (e.shiftKey) {
      // range select stub
      next.add(id);
    } else if (e.metaKey || e.ctrlKey) {
      if (next.has(id)) next.delete(id); else next.add(id);
    } else {
      next.clear(); next.add(id);
    }
    setSel(next);
  };

  return (
    <PaPanel
      title="Patch"
      subtitle={`${FIXTURES.length} fixtures · 2 universes`}
      style={{ width: 580 }}
      actions={
        <>
          <button className="btn small">
            <PaIcon name="plus" /> ADD
          </button>
          <button className="btn small ghost icon" title="Delete selected">
            <PaIcon name="trash" />
          </button>
        </>
      }
    >
      {/* Toolbar */}
      <div style={{ display: "flex", gap: 6, marginBottom: 8 }}>
        <div style={{ position: "relative", flex: 1 }}>
          <span style={{ position: "absolute", left: 7, top: "50%", transform: "translateY(-50%)", color: "var(--fg-muted)", display: "flex" }}>
            <PaIcon name="search" />
          </span>
          <input className="input" placeholder="Filter by name, type, address…" value={filter} onChange={(e) => setFilter(e.target.value)} style={{ paddingLeft: 24, height: 24 }} />
        </div>
        <button className="btn small">All</button>
        <button className="btn small ghost">Live</button>
        <button className="btn small ghost">U1</button>
        <button className="btn small ghost">U2</button>
      </div>

      {/* Header row */}
      <div style={{
        display: "grid",
        gridTemplateColumns: "32px 1fr 1.5fr 0.9fr 78px 32px",
        gap: 8,
        padding: "0 8px 6px 8px",
        borderBottom: "1px solid var(--border-soft)",
      }}>
        {["#", "Name", "Fixture Type", "Mode", "Address", ""].map((h, i) => (
          <span key={i} style={{
            fontSize: 9, fontWeight: 600, letterSpacing: "0.1em", textTransform: "uppercase",
            color: "var(--fg-muted)",
            textAlign: i === 4 ? "right" : i === 0 ? "right" : "left",
          }}>{h}</span>
        ))}
      </div>

      {/* Rows */}
      <div style={{ background: "var(--bg-input)", border: "1px solid var(--border-soft)", borderTop: "none", borderRadius: "0 0 3px 3px", maxHeight: 260, overflow: "auto" }}>
        {FIXTURES.map((f, i) => {
          const selected = sel.has(f.id);
          return (
            <div
              key={f.id}
              onClick={(e) => toggle(f.id, e)}
              style={{
                display: "grid",
                gridTemplateColumns: "32px 1fr 1.5fr 0.9fr 78px 32px",
                gap: 8, alignItems: "center",
                padding: "0 8px",
                height: 24,
                borderBottom: "1px solid oklch(0.18 0.005 240)",
                background: selected ? "oklch(0.24 0.04 215)" : (i % 2 === 0 ? "transparent" : "oklch(0.165 0.005 240 / 0.5)"),
                borderLeft: selected ? "2px solid var(--accent)" : "2px solid transparent",
                cursor: "pointer",
              }}
            >
              <span className="mono" style={{ fontSize: 11, color: selected ? "var(--accent)" : "var(--fg-muted)", textAlign: "right" }}>
                {String(f.id).padStart(3, "0")}
              </span>
              <span style={{ display: "flex", alignItems: "center", gap: 6, fontSize: 11, color: "var(--fg)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                <span className={`dot ${f.live ? "tx" : "idle"}`} />
                {f.name}
              </span>
              <span style={{ fontSize: 11, color: "var(--fg-secondary)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{f.type}</span>
              <span className="mono" style={{ fontSize: 10, color: "var(--fg-muted)" }}>{f.mode}</span>
              <span className="mono" style={{ fontSize: 11, color: selected ? "var(--fg)" : "var(--fg-secondary)", textAlign: "right" }}>
                {f.u}.{String(f.ch).padStart(3, "0")}
              </span>
              <span style={{ fontSize: 9, color: "var(--fg-faint)", textAlign: "right", fontFamily: "var(--font-mono)" }}>
                {f.live ? "OK" : "—"}
              </span>
            </div>
          );
        })}
      </div>

      {/* Footer */}
      <div style={{ display: "flex", alignItems: "center", gap: 12, padding: "8px 0 0 0", fontSize: 10, color: "var(--fg-muted)" }}>
        <span><span className="mono" style={{ color: "var(--accent)" }}>{sel.size}</span> selected</span>
        <span>·</span>
        <span><span className="mono" style={{ color: "var(--fg)" }}>{FIXTURES.length}</span> patched</span>
        <span>·</span>
        <span><span className="mono" style={{ color: "var(--rx)" }}>{FIXTURES.filter(f => f.live).length}</span> live</span>
        <span style={{ flex: 1 }} />
        <span className="mono" style={{ color: "var(--fg-faint)" }}>U1 81/512  ·  U2 65/512</span>
      </div>

      {/* Inline add fixture form */}
      <div style={{ marginTop: 12, padding: 10, background: "var(--bg-chrome)", border: "1px solid var(--border-soft)", borderRadius: 3 }}>
        <div style={{ display: "flex", alignItems: "center", marginBottom: 8 }}>
          <span className="eyebrow">Add Fixture</span>
          <span style={{ flex: 1 }} />
          <span className="mono" style={{ fontSize: 9, color: "var(--fg-faint)" }}>NEXT FREE: 2.078</span>
        </div>
        <div style={{ display: "grid", gridTemplateColumns: "1.4fr 1fr 1fr 80px 80px auto", gap: 6, alignItems: "end" }}>
          <div className="field">
            <span className="field-label">Type</span>
            <button className="input" style={{ display: "flex", alignItems: "center", justifyContent: "space-between", textAlign: "left" }}>
              <span>Martin · MAC Aura XB</span>
              <PaIcon name="chevron" />
            </button>
          </div>
          <div className="field">
            <span className="field-label">Mode</span>
            <button className="input" style={{ display: "flex", alignItems: "center", justifyContent: "space-between", textAlign: "left" }}>
              <span>16ch Extended</span>
              <PaIcon name="chevron" />
            </button>
          </div>
          <div className="field">
            <span className="field-label">Name</span>
            <input className="input" placeholder="Fixture 12" />
          </div>
          <div className="field">
            <span className="field-label">Univ</span>
            <input className="input numeric" defaultValue="2" />
          </div>
          <div className="field">
            <span className="field-label">Channel</span>
            <input className="input numeric" defaultValue="78" />
          </div>
          <button className="btn primary">
            <PaIcon name="plus" /> Patch
          </button>
        </div>
      </div>
    </PaPanel>
  );
}

window.PatchPanel = PatchPanel;
