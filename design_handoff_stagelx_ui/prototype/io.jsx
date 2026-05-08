/* global React, Atoms */
const { useState: useIoState } = React;
const { Panel: IoPanel, Icon: IoIcon } = Atoms;

const PROTOS = [
  { id: "artnet", label: "Art-Net",  status: "online",  rx: 1432, tx: 9821, hint: "U 0  ·  192.168.1.10" },
  { id: "sacn",   label: "sACN",     status: "online",  rx: 0,    tx: 4203, hint: "U 1  ·  pri 100" },
  { id: "usb",    label: "USB DMX",  status: "warning", rx: 0,    tx: 312,  hint: "Enttec /dev/cu…" },
  { id: "midi",   label: "MIDI",     status: "idle",    rx: 0,    tx: 0,    hint: "no port" },
  { id: "osc",    label: "OSC",      status: "online",  rx: 84,   tx: 0,    hint: ":7700  UDP" },
];

function IoPanelView() {
  const [active, setActive] = useIoState("artnet");

  return (
    <IoPanel
      title="DMX I/O"
      subtitle="3 active · 2 idle"
      style={{ width: 360 }}
    >
      {/* Protocol summary strip */}
      <div style={{
        display: "grid", gridTemplateColumns: "repeat(5, 1fr)", gap: 4,
        padding: 4, background: "var(--bg-input)", border: "1px solid var(--border-soft)", borderRadius: 3,
        marginBottom: 10,
      }}>
        {PROTOS.map((p) => (
          <button
            key={p.id}
            onClick={() => setActive(p.id)}
            style={{
              display: "flex", flexDirection: "column", alignItems: "center", gap: 4,
              padding: "6px 4px",
              borderRadius: 2,
              background: active === p.id ? "var(--bg-raised)" : "transparent",
              border: `1px solid ${active === p.id ? "var(--accent-dim)" : "transparent"}`,
              color: active === p.id ? "var(--fg)" : "var(--fg-secondary)",
            }}
          >
            <span className={`dot ${p.status === "online" ? "live" : p.status === "warning" ? "warn" : "idle"}`} />
            <span style={{ fontSize: 10, fontWeight: active === p.id ? 600 : 500, letterSpacing: "0.04em" }}>{p.label}</span>
          </button>
        ))}
      </div>

      {/* Active config */}
      {active === "artnet" && <ArtNetConfig />}
      {active === "sacn"   && <SacnConfig />}
      {active === "usb"    && <UsbConfig />}
      {active === "midi"   && <MidiConfig />}
      {active === "osc"    && <OscConfig />}

      {/* Live counters */}
      <div style={{
        marginTop: 12, padding: "8px 10px",
        background: "var(--bg-chrome)", border: "1px solid var(--border-soft)", borderRadius: 3,
        display: "grid", gridTemplateColumns: "1fr 1fr", gap: 8,
      }}>
        <Counter label="TX" value={(PROTOS.find(p => p.id === active)?.tx ?? 0).toLocaleString()} kind="tx" />
        <Counter label="RX" value={(PROTOS.find(p => p.id === active)?.rx ?? 0).toLocaleString()} kind="rx" />
      </div>
    </IoPanel>
  );
}

const Counter = ({ label, value, kind }) => (
  <div style={{ display: "flex", flexDirection: "column", gap: 2 }}>
    <div style={{ display: "flex", alignItems: "center", gap: 4 }}>
      <span className={`dot ${kind === "tx" ? "tx" : "live"}`} />
      <span style={{ fontSize: 9, fontWeight: 600, letterSpacing: "0.12em", color: "var(--fg-muted)", textTransform: "uppercase" }}>{label}</span>
    </div>
    <span className="mono" style={{ fontSize: 16, color: "var(--fg)", letterSpacing: "-0.01em" }}>{value}</span>
    <span className="mono" style={{ fontSize: 9, color: "var(--fg-faint)" }}>packets/s</span>
  </div>
);

const Row = ({ label, children }) => (
  <div style={{ display: "grid", gridTemplateColumns: "76px 1fr", gap: 8, alignItems: "center", marginBottom: 6 }}>
    <span style={{ fontSize: 10, color: "var(--fg-muted)", letterSpacing: "0.04em", textTransform: "uppercase", fontWeight: 500 }}>{label}</span>
    <div>{children}</div>
  </div>
);

const Toggle = ({ on, label }) => (
  <button style={{
    display: "inline-flex", alignItems: "center", gap: 6,
    padding: "0 8px", height: 22,
    background: on ? "oklch(0.30 0.06 215)" : "var(--bg-input)",
    border: `1px solid ${on ? "var(--accent-dim)" : "var(--border-soft)"}`,
    borderRadius: 3,
    fontSize: 10, fontWeight: 600, letterSpacing: "0.06em", textTransform: "uppercase",
    color: on ? "var(--accent)" : "var(--fg-muted)",
  }}>
    <span style={{
      width: 16, height: 8, borderRadius: 4,
      background: on ? "var(--accent)" : "var(--border)",
      position: "relative",
    }}>
      <span style={{
        position: "absolute", top: 1, left: on ? 9 : 1, width: 6, height: 6, borderRadius: "50%",
        background: on ? "white" : "var(--fg-muted)", transition: "left .15s",
      }} />
    </span>
    {label}
  </button>
);

function ArtNetConfig() {
  return (
    <div>
      <Row label="Bind"><input className="input mono" defaultValue="0.0.0.0" /></Row>
      <Row label="Dest"><input className="input mono" defaultValue="255.255.255.255" /></Row>
      <Row label="Universe">
        <div style={{ display: "flex", gap: 6 }}>
          <input className="input numeric" defaultValue="0" style={{ width: 64 }} />
          <span style={{ fontSize: 10, color: "var(--fg-faint)", alignSelf: "center", fontFamily: "var(--font-mono)" }}>0–32767</span>
        </div>
      </Row>
      <Row label="Receive">
        <div style={{ display: "flex", gap: 6, alignItems: "center" }}>
          <Toggle on={true} label="RX" />
          <input className="input mono" placeholder="any  (192.168.1.10,…)" style={{ flex: 1 }} />
        </div>
      </Row>
      <div style={{
        marginTop: 8, padding: "5px 8px",
        background: "var(--bg-input)", border: "1px solid var(--border-soft)", borderRadius: 3,
        display: "flex", alignItems: "center", gap: 6,
        fontSize: 10, color: "var(--fg-secondary)", fontFamily: "var(--font-mono)",
      }}>
        <span className="dot live" />
        bound 0.0.0.0:6454 · 2 nodes seen
      </div>
    </div>
  );
}

function SacnConfig() {
  return (
    <div>
      <Row label="Mode">
        <div style={{ display: "flex", gap: 4 }}>
          <Toggle on={true}  label="TX" />
          <Toggle on={false} label="RX" />
        </div>
      </Row>
      <Row label="Universe">
        <div style={{ display: "flex", gap: 6 }}>
          <input className="input numeric" defaultValue="1" style={{ width: 64 }} />
          <span style={{ fontSize: 10, color: "var(--fg-faint)", alignSelf: "center", fontFamily: "var(--font-mono)" }}>1–63999</span>
        </div>
      </Row>
      <Row label="Priority"><input className="input numeric" defaultValue="100" style={{ width: 64 }} /></Row>
      <Row label="Dest"><input className="input mono" defaultValue="239.255.0.1" /></Row>
    </div>
  );
}

function UsbConfig() {
  return (
    <div>
      <Row label="State"><Toggle on={true} label="TX ENABLED" /></Row>
      <Row label="Port">
        <div style={{ display: "flex", gap: 4 }}>
          <input className="input mono" defaultValue="/dev/cu.usbserial-EN386051" style={{ flex: 1 }} />
          <button className="btn small ghost icon"><IoIcon name="chevron" /></button>
        </div>
      </Row>
      <Row label="Universe"><input className="input numeric" defaultValue="1" style={{ width: 64 }} /></Row>
      <div style={{
        marginTop: 8, padding: "5px 8px",
        background: "oklch(0.22 0.05 80)", border: "1px solid var(--warning-dim)", borderRadius: 3,
        display: "flex", alignItems: "center", gap: 6,
        fontSize: 10, color: "var(--warning)", fontFamily: "var(--font-mono)",
      }}>
        <span className="dot warn" />
        port busy — close other apps using this device
      </div>
    </div>
  );
}

function MidiConfig() {
  const ccs = [["Dimmer", 1], ["Pan", 2], ["Tilt", 3], ["Zoom", 4], ["Red", 5], ["Green", 6], ["Blue", 7], ["Strobe", 8]];
  return (
    <div>
      <Row label="State"><Toggle on={false} label="ENABLE" /></Row>
      <Row label="Port">
        <div style={{ display: "flex", gap: 4 }}>
          <input className="input mono" placeholder="select MIDI input…" style={{ flex: 1 }} />
          <button className="btn small ghost icon"><IoIcon name="chevron" /></button>
        </div>
      </Row>
      <div style={{ marginTop: 10 }}>
        <div style={{ display: "flex", alignItems: "baseline", gap: 8, marginBottom: 6 }}>
          <span className="eyebrow">CC Mapping</span>
          <span style={{ flex: 1 }} />
          <button className="btn ghost small">Learn</button>
        </div>
        <div style={{ display: "grid", gridTemplateColumns: "repeat(2, 1fr)", gap: 4 }}>
          {ccs.map(([label, cc]) => (
            <div key={label} style={{
              display: "grid", gridTemplateColumns: "1fr auto", gap: 6, alignItems: "center",
              padding: "4px 8px", background: "var(--bg-input)", border: "1px solid var(--border-soft)", borderRadius: 2,
            }}>
              <span style={{ fontSize: 10, color: "var(--fg-secondary)" }}>{label}</span>
              <span className="mono" style={{ fontSize: 10, color: "var(--fg)" }}>CC {String(cc).padStart(3, "0")}</span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

function OscConfig() {
  return (
    <div>
      <Row label="State"><Toggle on={true} label="LISTENING" /></Row>
      <Row label="Port"><input className="input numeric" defaultValue="7700" style={{ width: 64 }} /></Row>
      <div style={{
        marginTop: 8, padding: "8px 10px",
        background: "var(--bg-input)", border: "1px solid var(--border-soft)", borderRadius: 3,
        display: "flex", flexDirection: "column", gap: 4,
      }}>
        <span className="eyebrow" style={{ fontSize: 9 }}>Address Pattern</span>
        <span className="mono" style={{ fontSize: 11, color: "var(--accent)" }}>/fixture/&#123;id&#125;/&#123;attr&#125;</span>
        <span className="mono" style={{ fontSize: 9, color: "var(--fg-muted)" }}>f32 · 0.0–1.0 normalised</span>
      </div>
    </div>
  );
}

window.IoPanelView = IoPanelView;
