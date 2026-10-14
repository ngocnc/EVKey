// Liquid-glass look shared by the settings and popup windows (same origin, so the same local storage).
// Look-only preferences, applied as CSS variables: --t (glass fill), --blur, --tint (colour or gradient), --ts (tint strength).
const GLASS_DEFAULTS = { style: "regular", tint: "", "tint-end": "", strength: "30", accent: "" };
const glassGet = (key) => localStorage.getItem("evkey-glass-" + key) ?? GLASS_DEFAULTS[key];

// Partner colour for a gradient's second stop: hue shifted slightly, saturation and brightness floored.
function hsvToHex(h, s, v) {
  const i = Math.floor(h * 6), f = h * 6 - i;
  const p = v * (1 - s), q = v * (1 - f * s), t = v * (1 - (1 - f) * s);
  const [r, g, b] = [[v, t, p], [q, v, p], [p, v, t], [p, q, v], [t, p, v], [v, p, q]][i % 6];
  return "#" + [r, g, b].map((x) => Math.round(x * 255).toString(16).padStart(2, "0")).join("");
}
function companionTint(hex) {
  const n = parseInt(hex.slice(1), 16);
  const [r, g, b] = [n >> 16, n >> 8, n].map((x) => (x & 255) / 255);
  const max = Math.max(r, g, b), d = max - Math.min(r, g, b);
  let h = 0;
  if (d) h = (max === r ? (g - b) / d + (g < b ? 6 : 0) : max === g ? (b - r) / d + 2 : (r - g) / d + 4) / 6;
  return hsvToHex((h + 0.12) % 1, Math.max(max ? d / max : 0, 0.5), Math.max(max, 0.6));
}

function applyGlassLook() {
  const root = document.documentElement.style;
  const clear = glassGet("style") === "clear";
  root.setProperty("--t", clear ? 1 : 0.5);
  root.setProperty("--blur", clear ? "10px" : "28px");
  const [from, to] = [glassGet("tint"), glassGet("tint-end")];
  root.setProperty("--tint", from ? (to ? `linear-gradient(135deg, ${from}, ${to})` : from) : "rgb(var(--base))");
  root.setProperty("--ts", glassGet("strength") / 100);
  // Accent colour (logo, switches, sliders, scrollbar): empty keeps the stylesheet's pink-to-peach pair.
  const accent = glassGet("accent");
  if (accent) {
    root.setProperty("--accent", accent);
    root.setProperty("--accent-2", companionTint(accent));
  } else {
    root.removeProperty("--accent");
    root.removeProperty("--accent-2");
  }
}

applyGlassLook();
window.addEventListener("storage", (e) => e.key?.startsWith("evkey-glass-") && applyGlassLook());
