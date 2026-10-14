// Liquid-glass look shared by the settings and popup windows (same origin, so the same local storage).
// Look-only preferences, applied as CSS variables: --t (glass fill), --blur, --tint (colour or gradient), --ts (tint strength).
const GLASS_DEFAULTS = { style: "regular", tint: "", "tint-end": "", strength: "30" };
const glassGet = (key) => localStorage.getItem("evkey-glass-" + key) ?? GLASS_DEFAULTS[key];

function applyGlassLook() {
  const root = document.documentElement.style;
  const clear = glassGet("style") === "clear";
  root.setProperty("--t", clear ? 1 : 0.5);
  root.setProperty("--blur", clear ? "10px" : "28px");
  const [from, to] = [glassGet("tint"), glassGet("tint-end")];
  root.setProperty("--tint", from ? (to ? `linear-gradient(135deg, ${from}, ${to})` : from) : "rgb(var(--base))");
  root.setProperty("--ts", glassGet("strength") / 100);
}

applyGlassLook();
window.addEventListener("storage", (e) => e.key?.startsWith("evkey-glass-") && applyGlassLook());
