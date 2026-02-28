/**
 * Basement Garage Door Card
 * A custom Lovelace card for controlling and visualizing the basement garage door.
 *
 * Configuration:
 *   type: custom:basement-garage-card
 *   state_entity: sensor.basement_gdopener_state   (or whatever HA named it)
 *   title: "Basement Garage"                        (optional)
 */

const STATES = {
  open: { label: "Open", color: "#f97316", bg: "#1a0f00" },
  closed: { label: "Closed", color: "#22c55e", bg: "#001a08" },
  opening: { label: "Opening…", color: "#facc15", bg: "#1a1500" },
  closing: { label: "Closing…", color: "#facc15", bg: "#1a1500" },
  "safety-stopped-opening": {
    label: "Safety Stop (was opening)",
    color: "#ef4444",
    bg: "#1a0000",
  },
  "safety-stopped-closing": {
    label: "Safety Stop (was closing)",
    color: "#ef4444",
    bg: "#1a0000",
  },
  unknown: { label: "Unknown", color: "#94a3b8", bg: "#0f1117" },
};

const STYLE = `
  @import url('https://fonts.googleapis.com/css2?family=Share+Tech+Mono&family=Barlow+Condensed:wght@300;600;800&display=swap');

  :host {
    display: block;
  }

  .card {
    font-family: 'Barlow Condensed', sans-serif;
    background: #0a0d14;
    border-radius: 16px;
    overflow: hidden;
    padding: 0;
    box-shadow: 0 0 0 1px rgba(255,255,255,0.06), 0 8px 40px rgba(0,0,0,0.6);
    position: relative;
    user-select: none;
  }

  .noise {
    position: absolute;
    inset: 0;
    background-image: url("data:image/svg+xml,%3Csvg viewBox='0 0 200 200' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='n'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.9' numOctaves='4' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23n)' opacity='0.04'/%3E%3C/svg%3E");
    pointer-events: none;
    z-index: 0;
    border-radius: 16px;
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 14px 20px 10px;
    position: relative;
    z-index: 1;
  }

  .title {
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 3px;
    text-transform: uppercase;
    color: rgba(255,255,255,0.35);
    font-family: 'Share Tech Mono', monospace;
  }

  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: #22c55e;
    box-shadow: 0 0 8px #22c55e;
    transition: background 0.4s, box-shadow 0.4s;
  }
  .status-dot.offline {
    background: #ef4444;
    box-shadow: 0 0 8px #ef4444;
  }
  .status-dot.pulse {
    animation: pulse 2s ease-in-out infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.3; }
  }

  /* ── GARAGE ILLUSTRATION ── */
  .scene {
    position: relative;
    z-index: 1;
    display: flex;
    justify-content: center;
    align-items: flex-end;
    padding: 10px 20px 0;
    height: 160px;
  }

  .garage-wrap {
    position: relative;
    width: 200px;
    height: 140px;
  }

  /* Building */
  .building {
    position: absolute;
    bottom: 0;
    left: 0; right: 0;
    height: 110px;
    background: linear-gradient(170deg, #1e2533, #141820);
    border-radius: 4px 4px 0 0;
    border: 1px solid rgba(255,255,255,0.08);
    border-bottom: none;
  }

  /* Roof triangle */
  .roof {
    position: absolute;
    top: -28px;
    left: -6px; right: -6px;
    height: 32px;
    overflow: hidden;
  }
  .roof::before {
    content: '';
    display: block;
    width: 0; height: 0;
    border-left: 106px solid transparent;
    border-right: 106px solid transparent;
    border-bottom: 32px solid #1e2533;
    filter: drop-shadow(0 -2px 4px rgba(0,0,0,0.5));
  }

  /* Door opening (the dark hole) */
  .door-frame {
    position: absolute;
    bottom: 0;
    left: 50%;
    transform: translateX(-50%);
    width: 120px;
    height: 86px;
    background: #04060a;
    border-radius: 2px 2px 0 0;
    border: 1px solid rgba(255,255,255,0.1);
    border-bottom: none;
    overflow: hidden;
  }

  /* Interior glow when open */
  .door-interior {
    position: absolute;
    inset: 0;
    background: radial-gradient(ellipse at 50% 100%, var(--glow-color, transparent) 0%, transparent 70%);
    transition: background 0.6s ease;
    z-index: 0;
  }

  /* The door panel itself */
  .door-panel {
    position: absolute;
    left: 0; right: 0;
    top: 0;
    height: 86px;
    background: linear-gradient(180deg, #2c3347, #232838);
    border-bottom: 2px solid rgba(255,255,255,0.05);
    transform-origin: top center;
    transform: translateY(0%);
    transition: transform 0.7s cubic-bezier(0.4, 0, 0.2, 1);
    z-index: 2;
  }

  /* Door panel stripes */
  .door-panel::before {
    content: '';
    position: absolute;
    inset: 0;
    background: repeating-linear-gradient(
      180deg,
      transparent 0px,
      transparent 19px,
      rgba(255,255,255,0.05) 19px,
      rgba(255,255,255,0.05) 21px
    );
  }

  /* Door panel sheen */
  .door-panel::after {
    content: '';
    position: absolute;
    inset: 0;
    background: linear-gradient(90deg, transparent 0%, rgba(255,255,255,0.03) 30%, transparent 60%);
  }

  /* Door states */
  .door-panel.open {
    transform: translateY(-100%);
  }
  .door-panel.opening {
    animation: door-open-anim 1.4s ease-in-out infinite alternate;
  }
  .door-panel.closing {
    animation: door-close-anim 1.4s ease-in-out infinite alternate;
  }
  .door-panel.safety-stopped-opening {
    transform: translateY(-45%);
  }
  .door-panel.safety-stopped-closing {
    transform: translateY(-45%);
  }

  @keyframes door-open-anim {
    from { transform: translateY(-10%); }
    to   { transform: translateY(-90%); }
  }
  @keyframes door-close-anim {
    from { transform: translateY(-90%); }
    to   { transform: translateY(-10%); }
  }

  /* Ground line */
  .ground {
    position: absolute;
    bottom: 0; left: -10px; right: -10px;
    height: 3px;
    background: linear-gradient(90deg, transparent, rgba(255,255,255,0.08), transparent);
  }

  /* Driveway approach marks */
  .driveway {
    position: absolute;
    bottom: -14px;
    left: 50%;
    transform: translateX(-50%);
    width: 80px;
    height: 12px;
    display: flex;
    gap: 6px;
    justify-content: center;
  }
  .driveway span {
    display: block;
    flex: 1;
    background: rgba(255,255,255,0.05);
    border-radius: 1px;
  }

  /* ── STATE LABEL ── */
  .state-display {
    position: relative;
    z-index: 1;
    text-align: center;
    padding: 18px 20px 8px;
  }

  .state-label {
    font-size: 36px;
    font-weight: 800;
    letter-spacing: 1px;
    line-height: 1;
    color: var(--state-color, #94a3b8);
    transition: color 0.5s ease;
    text-transform: uppercase;
  }

  .state-sublabel {
    font-family: 'Share Tech Mono', monospace;
    font-size: 10px;
    letter-spacing: 2px;
    color: rgba(255,255,255,0.2);
    margin-top: 4px;
    text-transform: uppercase;
  }

  /* ── CONTROLS ── */
  .controls {
    position: relative;
    z-index: 1;
    display: flex;
    gap: 10px;
    padding: 14px 20px 20px;
  }

  .btn {
    flex: 1;
    padding: 12px;
    border: none;
    border-radius: 10px;
    font-family: 'Barlow Condensed', sans-serif;
    font-size: 17px;
    font-weight: 700;
    letter-spacing: 2px;
    text-transform: uppercase;
    cursor: pointer;
    transition: all 0.18s ease;
    position: relative;
    overflow: hidden;
  }

  .btn::after {
    content: '';
    position: absolute;
    inset: 0;
    background: rgba(255,255,255,0);
    transition: background 0.15s;
  }
  .btn:active::after {
    background: rgba(255,255,255,0.1);
  }

  .btn-open {
    background: linear-gradient(135deg, #9a3412, #ea580c);
    color: #fff;
    box-shadow: 0 4px 20px rgba(234,88,12,0.3);
  }
  .btn-open:hover:not(:disabled) {
    box-shadow: 0 4px 28px rgba(234,88,12,0.5);
    transform: translateY(-1px);
  }

  .btn-close {
    background: linear-gradient(135deg, #14532d, #16a34a);
    color: #fff;
    box-shadow: 0 4px 20px rgba(22,163,74,0.25);
  }
  .btn-close:hover:not(:disabled) {
    box-shadow: 0 4px 28px rgba(22,163,74,0.45);
    transform: translateY(-1px);
  }

  .btn:disabled {
    opacity: 0.3;
    cursor: not-allowed;
    transform: none;
  }

  /* Safety stop warning */
  .safety-banner {
    position: relative;
    z-index: 1;
    margin: 0 20px;
    padding: 8px 12px;
    background: rgba(239,68,68,0.12);
    border: 1px solid rgba(239,68,68,0.3);
    border-radius: 8px;
    font-family: 'Share Tech Mono', monospace;
    font-size: 10px;
    letter-spacing: 1.5px;
    color: #ef4444;
    text-align: center;
    text-transform: uppercase;
    display: none;
  }
  .safety-banner.visible {
    display: block;
    animation: blink-border 1.2s ease-in-out infinite;
  }

  @keyframes blink-border {
    0%, 100% { border-color: rgba(239,68,68,0.3); }
    50% { border-color: rgba(239,68,68,0.8); }
  }

  .last-seen {
    position: relative;
    z-index: 1;
    text-align: center;
    padding: 8px 20px 16px;
    font-family: 'Share Tech Mono', monospace;
    font-size: 10px;
    color: rgba(255,255,255,0.15);
    letter-spacing: 1px;
  }
`;

class BasementGarageCard extends HTMLElement {
  constructor() {
    super();
    this.attachShadow({ mode: "open" });
    this._hass = null;
    this._config = {};
    this._rendered = false;
  }

  setConfig(config) {
    if (!config.state_entity) {
      throw new Error("Please define state_entity in card config");
    }
    this._config = {
      title: config.title || "Garage Door",
      state_entity: config.state_entity,
      ...config,
    };
  }

  set hass(hass) {
    this._hass = hass;
    if (!this._rendered) {
      this._render();
      this._rendered = true;
    }
    this._update();
  }

  _render() {
    const shadow = this.shadowRoot;
    shadow.innerHTML = `
      <style>${STYLE}</style>
      <ha-card>
        <div class="card">
          <div class="noise"></div>

          <div class="header">
            <span class="title">${this._config.title}</span>
            <div class="status-dot offline" id="dot"></div>
          </div>

          <div class="scene">
            <div class="garage-wrap">
              <div class="roof"></div>
              <div class="building">
                <div class="door-frame">
                  <div class="door-interior" id="interior"></div>
                  <div class="door-panel closed" id="door"></div>
                </div>
              </div>
              <div class="ground"></div>
              <div class="driveway">
                <span></span><span></span><span></span>
              </div>
            </div>
          </div>

          <div class="state-display">
            <div class="state-label" id="state-label">—</div>
            <div class="state-sublabel" id="state-sub">awaiting data</div>
          </div>

          <div class="safety-banner" id="safety-banner">
            ⚠ Safety stop triggered — check door before proceeding
          </div>

          <div class="controls">
            <button class="btn btn-open" id="btn-open" disabled>Open</button>
            <button class="btn btn-close" id="btn-close" disabled>Close</button>
          </div>

          <div class="last-seen" id="last-seen"></div>
        </div>
      </ha-card>
    `;

    shadow
      .getElementById("btn-open")
      .addEventListener("click", () => this._command("open"));
    shadow
      .getElementById("btn-close")
      .addEventListener("click", () => this._command("close"));
  }

  _update() {
    const hass = this._hass;
    const s = hass.shadowRoot;
    const shadow = this.shadowRoot;

    const stateObj = hass.states[this._config.state_entity];
    const rawState = stateObj ? stateObj.state : "unknown";
    const stateInfo = STATES[rawState] || STATES["unknown"];

    // Door animation
    const door = shadow.getElementById("door");
    door.className = `door-panel ${rawState}`;

    // Interior glow
    const interior = shadow.getElementById("interior");
    const glowMap = {
      open: "rgba(250,180,80,0.2)",
      opening: "rgba(250,180,80,0.12)",
      closing: "rgba(250,180,80,0.08)",
      "safety-stopped-opening": "rgba(239,68,68,0.15)",
      "safety-stopped-closing": "rgba(239,68,68,0.15)",
    };
    interior.style.setProperty(
      "--glow-color",
      glowMap[rawState] || "transparent",
    );
    interior.style.background = `radial-gradient(ellipse at 50% 100%, ${glowMap[rawState] || "transparent"} 0%, transparent 70%)`;

    // State label
    const label = shadow.getElementById("state-label");
    label.textContent = stateInfo.label;
    label.style.setProperty("--state-color", stateInfo.color);
    label.style.color = stateInfo.color;

    // Sublabel
    const sub = shadow.getElementById("state-sub");
    sub.textContent =
      stateObj?.state === "unavailable"
        ? "no signal"
        : "basement_gdopener/state";

    // Online dot
    const dot = shadow.getElementById("dot");
    // "unknown" is a valid state published by the device (e.g. during init/transition).
    // Only HA's own "unavailable" means the entity is truly unreachable.
    const isOnline = stateObj && stateObj.state !== "unavailable";
    const isMoving = rawState === "opening" || rawState === "closing";
    dot.className = `status-dot${isOnline ? "" : " offline"}${isMoving ? " pulse" : ""}`;

    // Safety banner
    const banner = shadow.getElementById("safety-banner");
    const isSafety =
      rawState === "safety-stopped-opening" ||
      rawState === "safety-stopped-closing";
    banner.className = `safety-banner${isSafety ? " visible" : ""}`;

    // Buttons
    const btnOpen = shadow.getElementById("btn-open");
    const btnClose = shadow.getElementById("btn-close");

    // Can't open if already open or opening; can't close if closed or closing
    const canOpen = !["open", "opening"].includes(rawState) && isOnline;
    const canClose = !["closed", "closing"].includes(rawState) && isOnline;
    btnOpen.disabled = !canOpen;
    btnClose.disabled = !canClose;

    // Last seen
    if (stateObj && stateObj.last_changed) {
      const ago = this._timeAgo(new Date(stateObj.last_changed));
      shadow.getElementById("last-seen").textContent = `updated ${ago}`;
    }
  }

  _command(cmd) {
    this._hass.callService("mqtt", "publish", {
      topic: "basement_gdopener/command",
      payload: cmd,
    });

    // Visual feedback: briefly disable both buttons
    const shadow = this.shadowRoot;
    const btnOpen = shadow.getElementById("btn-open");
    const btnClose = shadow.getElementById("btn-close");
    btnOpen.disabled = true;
    btnClose.disabled = true;
    setTimeout(() => this._update(), 1200);
  }

  _timeAgo(date) {
    const secs = Math.floor((Date.now() - date) / 1000);
    if (secs < 10) return "just now";
    if (secs < 60) return `${secs}s ago`;
    if (secs < 3600) return `${Math.floor(secs / 60)}m ago`;
    return `${Math.floor(secs / 3600)}h ago`;
  }

  getCardSize() {
    return 4;
  }

  static getConfigElement() {
    return document.createElement("div"); // no visual editor
  }

  static getStubConfig() {
    return { state_entity: "sensor.basement_gdopener_state" };
  }
}

customElements.define("basement-garage-card", BasementGarageCard);

window.customCards = window.customCards || [];
window.customCards.push({
  type: "basement-garage-card",
  name: "Basement Garage Door",
  description: "Animated garage door card with open/close controls",
  preview: true,
});
