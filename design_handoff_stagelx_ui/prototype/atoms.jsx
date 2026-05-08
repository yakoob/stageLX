/* global React */
const { useState } = React;

// ── Atoms ────────────────────────────────────────────────────────────

// Tiny inline icons
const Icon = ({ name, size = 12 }) => {
  const s = { width: size, height: size, display: "inline-block", flexShrink: 0 };
  const stroke = "currentColor", sw = 1.5;
  switch (name) {
    case "x":
      return <svg style={s} viewBox="0 0 16 16" fill="none"><path d="M3 3l10 10M13 3L3 13" stroke={stroke} strokeWidth={sw}/></svg>;
    case "minimize":
      return <svg style={s} viewBox="0 0 16 16" fill="none"><path d="M3 8h10" stroke={stroke} strokeWidth={sw}/></svg>;
    case "expand":
      return <svg style={s} viewBox="0 0 16 16" fill="none"><path d="M3 3h4M3 3v4M13 13H9M13 13V9" stroke={stroke} strokeWidth={sw}/></svg>;
    case "detach":
      return <svg style={s} viewBox="0 0 16 16" fill="none"><path d="M6 3H3v3M3 10v3h3M10 3h3v3M13 10v3h-3" stroke={stroke} strokeWidth={sw}/></svg>;
    case "search":
      return <svg style={s} viewBox="0 0 16 16" fill="none"><circle cx="7" cy="7" r="4" stroke={stroke} strokeWidth={sw}/><path d="M10 10l3 3" stroke={stroke} strokeWidth={sw}/></svg>;
    case "plus":
      return <svg style={s} viewBox="0 0 16 16" fill="none"><path d="M8 3v10M3 8h10" stroke={stroke} strokeWidth={sw}/></svg>;
    case "trash":
      return <svg style={s} viewBox="0 0 16 16" fill="none"><path d="M3 4h10M5 4V3a1 1 0 011-1h4a1 1 0 011 1v1M6 7v5M10 7v5M4 4l1 9a1 1 0 001 1h4a1 1 0 001-1l1-9" stroke={stroke} strokeWidth={sw}/></svg>;
    case "folder":
      return <svg style={s} viewBox="0 0 16 16" fill="none"><path d="M2 5a1 1 0 011-1h3l1 1h6a1 1 0 011 1v6a1 1 0 01-1 1H3a1 1 0 01-1-1V5z" stroke={stroke} strokeWidth={sw}/></svg>;
    case "chevron":
      return <svg style={s} viewBox="0 0 16 16" fill="none"><path d="M5 6l3 3 3-3" stroke={stroke} strokeWidth={sw}/></svg>;
    case "lock":
      return <svg style={s} viewBox="0 0 16 16" fill="none"><rect x="3" y="7" width="10" height="7" rx="1" stroke={stroke} strokeWidth={sw}/><path d="M5 7V5a3 3 0 016 0v2" stroke={stroke} strokeWidth={sw}/></svg>;
    case "wave":
      return <svg style={s} viewBox="0 0 16 16" fill="none"><path d="M1 8c2-4 3-4 4 0s2 4 3 0 2-4 3 0 2 4 4 0" stroke={stroke} strokeWidth={sw}/></svg>;
    case "dot":
      return <svg style={s} viewBox="0 0 16 16"><circle cx="8" cy="8" r="2.5" fill={stroke}/></svg>;
    default: return null;
  }
};

// Panel chrome
const Panel = ({ title, subtitle, actions, children, style }) => (
  <div className="panel" style={style}>
    <div className="panel-titlebar">
      <span className="title">{title}</span>
      {subtitle && <span style={{ fontSize: 10, color: "var(--fg-muted)", fontFamily: "var(--font-mono)" }}>{subtitle}</span>}
      <span className="grow" />
      {actions}
      <button className="icon-btn" title="Detach"><Icon name="detach" /></button>
      <button className="icon-btn" title="Minimize"><Icon name="minimize" /></button>
    </div>
    <div className="panel-body">{children}</div>
  </div>
);

// Section header inside a panel
const Section = ({ label, hint, children, action }) => (
  <div style={{ marginBottom: 14 }}>
    <div style={{ display: "flex", alignItems: "baseline", gap: 8, marginBottom: 8 }}>
      <span className="eyebrow">{label}</span>
      {hint && <span style={{ fontSize: 10, color: "var(--fg-faint)", fontFamily: "var(--font-mono)" }}>{hint}</span>}
      <span style={{ flex: 1 }} />
      {action}
    </div>
    {children}
  </div>
);

// Encoder dial — circular knob with arc fill, big numeric readout
const Encoder = ({ label, value, unit = "", min = 0, max = 100, decimals = 0, sub, size = 76, accent = "var(--accent)" }) => {
  const norm = (value - min) / (max - min);
  const start = -135, end = 135;
  const angle = start + norm * (end - start);
  const r = size / 2 - 6;
  const cx = size / 2, cy = size / 2;
  // Build arc path
  const arc = (a0, a1) => {
    const rad = (a) => ((a - 90) * Math.PI) / 180;
    const x0 = cx + r * Math.cos(rad(a0));
    const y0 = cy + r * Math.sin(rad(a0));
    const x1 = cx + r * Math.cos(rad(a1));
    const y1 = cy + r * Math.sin(rad(a1));
    const large = Math.abs(a1 - a0) > 180 ? 1 : 0;
    const sweep = a1 > a0 ? 1 : 0;
    return `M ${x0} ${y0} A ${r} ${r} 0 ${large} ${sweep} ${x1} ${y1}`;
  };
  const indRad = ((angle - 90) * Math.PI) / 180;
  const indX = cx + (r - 3) * Math.cos(indRad);
  const indY = cy + (r - 3) * Math.sin(indRad);
  const innerR = r - 7;

  return (
    <div style={{ display: "flex", flexDirection: "column", alignItems: "center", gap: 4 }}>
      <div style={{ position: "relative", width: size, height: size }}>
        <svg width={size} height={size} style={{ display: "block" }}>
          {/* track */}
          <path d={arc(start, end)} stroke="var(--border)" strokeWidth="2" fill="none" strokeLinecap="round" />
          {/* fill */}
          <path d={arc(start, angle)} stroke={accent} strokeWidth="2" fill="none" strokeLinecap="round" />
          {/* hub */}
          <circle cx={cx} cy={cy} r={innerR} fill="var(--bg-input)" stroke="var(--border)" strokeWidth="1" />
          {/* indicator dot */}
          <circle cx={indX} cy={indY} r="2.5" fill={accent} />
        </svg>
        <div style={{ position: "absolute", inset: 0, display: "flex", flexDirection: "column", alignItems: "center", justifyContent: "center", pointerEvents: "none" }}>
          <span className="mono" style={{ fontSize: 18, fontWeight: 500, color: "var(--fg)", lineHeight: 1, letterSpacing: "-0.02em" }}>
            {value.toFixed(decimals)}{unit && <span style={{ fontSize: 10, color: "var(--fg-muted)", marginLeft: 1 }}>{unit}</span>}
          </span>
          {sub && <span className="mono" style={{ fontSize: 9, color: "var(--fg-muted)", marginTop: 2 }}>{sub}</span>}
        </div>
      </div>
      <div style={{ fontSize: 10, color: "var(--fg-secondary)", letterSpacing: "0.06em", textTransform: "uppercase", fontWeight: 500 }}>{label}</div>
    </div>
  );
};

// Vertical fader
const Fader = ({ label, value, height = 130, accent = "var(--accent)", unit = "%" }) => {
  const pct = value;
  return (
    <div style={{ display: "flex", flexDirection: "column", alignItems: "center", gap: 6 }}>
      <span className="mono" style={{ fontSize: 14, fontWeight: 500, color: "var(--fg)", letterSpacing: "-0.01em" }}>
        {Math.round(value)}<span style={{ fontSize: 10, color: "var(--fg-muted)" }}>{unit}</span>
      </span>
      <div style={{ position: "relative", width: 28, height, background: "var(--bg-input)", borderRadius: 3, border: "1px solid var(--border)" }}>
        {/* tick marks */}
        {[0, 25, 50, 75, 100].map((t) => (
          <div key={t} style={{ position: "absolute", right: -6, top: `calc(${100 - t}% - 0.5px)`, width: 4, height: 1, background: "var(--border)" }} />
        ))}
        {/* fill */}
        <div style={{ position: "absolute", left: 1, right: 1, bottom: 1, height: `calc(${pct}% - 2px)`, background: `linear-gradient(180deg, ${accent} 0%, oklch(0.55 0.10 215) 100%)`, borderRadius: 2, opacity: 0.8 }} />
        {/* cap */}
        <div style={{ position: "absolute", left: -3, right: -3, top: `calc(${100 - pct}% - 7px)`, height: 14, background: "linear-gradient(180deg, oklch(0.42 0.005 240) 0%, oklch(0.30 0.005 240) 100%)", border: "1px solid var(--border-strong)", borderRadius: 2, boxShadow: "0 2px 4px rgba(0,0,0,0.4)" }}>
          <div style={{ position: "absolute", left: 2, right: 2, top: "50%", height: 1, background: accent, opacity: 0.9 }} />
        </div>
      </div>
      <div style={{ fontSize: 10, color: "var(--fg-secondary)", letterSpacing: "0.06em", textTransform: "uppercase", fontWeight: 500 }}>{label}</div>
    </div>
  );
};

// Color swatch
const Swatch = ({ color, label, selected, onClick }) => (
  <button
    onClick={onClick}
    style={{
      display: "flex", flexDirection: "column", alignItems: "center", gap: 3,
      padding: 4, borderRadius: 3,
      background: selected ? "var(--bg-raised)" : "transparent",
      border: `1px solid ${selected ? "var(--accent-dim)" : "transparent"}`,
      width: 42,
    }}
  >
    <div style={{
      width: 28, height: 18, borderRadius: 2,
      background: color,
      border: "1px solid oklch(0 0 0 / 0.4)",
      boxShadow: selected ? `0 0 0 1px var(--accent)` : "none",
    }} />
    <span style={{ fontSize: 9, color: selected ? "var(--fg)" : "var(--fg-muted)", letterSpacing: "0.02em" }}>{label}</span>
  </button>
);

// Tab
const Tab = ({ children, active, onClick, badge }) => (
  <button
    onClick={onClick}
    style={{
      height: 26,
      padding: "0 10px",
      background: active ? "var(--bg-panel)" : "transparent",
      borderTop: active ? "1px solid var(--accent-dim)" : "1px solid transparent",
      color: active ? "var(--fg)" : "var(--fg-secondary)",
      fontSize: 11,
      fontWeight: active ? 600 : 500,
      letterSpacing: "0.02em",
      display: "inline-flex",
      alignItems: "center",
      gap: 6,
    }}
  >
    {children}
    {badge != null && <span className="mono" style={{ fontSize: 9, color: active ? "var(--accent)" : "var(--fg-muted)" }}>{badge}</span>}
  </button>
);

window.Atoms = { Icon, Panel, Section, Encoder, Fader, Swatch, Tab };
