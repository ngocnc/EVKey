const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const $ = (id) => document.getElementById(id);

const applyTheme = () => {
  const th = localStorage.getItem("evkey-theme") ?? "system";
  window.__TAURI__.window.getCurrentWindow().setTheme(th === "system" ? null : th).catch(console.error);
};
applyTheme();

const menu = $("ac-menu");
const setup = menu.querySelector(".ac-setup");
// Menu buttons carry the action id actions.rs expects; a click runs it and must not close the panel.
for (const btn of menu.querySelectorAll("[data-id]")) {
  btn.onclick = (e) => {
    e.stopPropagation();
    invoke("actions_run", { action: btn.dataset.id });
  };
}
// Shown when no Gemini key is saved: opens the Translate tab where the key is entered.
setup.onclick = () => invoke("popup_open_settings", { tab: "translate" });

let fill = null;

async function render() {
  const v = await invoke("actions_state");
  setUiLanguage(v.ui_language);
  const menuOpen = v.status === "menu";
  const loading = v.status === "loading";
  for (const el of menu.querySelectorAll("[data-key]")) (el.querySelector("span") ?? el).textContent = t(el.dataset.key);
  for (const btn of menu.querySelectorAll("[data-ai]")) btn.disabled = !v.gemini;
  setup.hidden = v.gemini;
  menu.hidden = !menuOpen;
  const text = $("tr-text");
  text.hidden = menuOpen;
  // While loading, v.text is the action id (actions.rs perform); grouped ids share one loading text.
  const group = /^(rewrite|convert|case|strip)/.exec(v.text)?.[1];
  const loadingKey = group === "case" || group === "strip" ? "text" : group ?? v.text;
  text.textContent = loading ? t("actions.loading." + loadingKey) : v.text;
  text.className = "tr-text " + (loading ? "loading" : v.status === "error" ? "error" : "");
  const copy = $("ac-copy");
  copy.hidden = v.status !== "ok";
  copy.title = t(v.replace ? "actions.replace" : "actions.copy");
  copy.dataset.cmd = v.replace ? "actions_replace" : "actions_copy";
  $("ac-copy-icon").hidden = v.replace;
  $("ac-replace-icon").hidden = !v.replace;
  // The countdown bar starts when the panel is shown and runs for the configured time.
  fill?.cancel();
  // Hidden while loading and when the panel never closes by itself; it keeps its space so the content does not jump.
  const timed = !loading && !menuOpen && v.ms > 0;
  $("tr-bar").style.visibility = timed ? "visible" : "hidden";
  if (timed) fill = $("tr-fill").animate([{ transform: "scaleX(1)" }, { transform: "scaleX(0)" }], { duration: v.ms, easing: "linear", fill: "forwards" });
  requestAnimationFrame(() => invoke("actions_fit", { height: Math.ceil($("panel").getBoundingClientRect().height) }));
}

// Copies the result (translate hotkey) or replaces the selection with it (action menu), then hides the panel.
$("ac-copy").onclick = (e) => {
  e.stopPropagation();
  invoke(e.currentTarget.dataset.cmd);
};

// The panel never takes focus, so there is no Esc. The x button closes it; a plain click on it does too.
// Dragging moves it and keeps it open (the auto-hide timer and countdown stop).
const win = window.__TAURI__.window.getCurrentWindow();
let dragFrom = null;
$("panel").addEventListener("mousedown", (e) => {
  if (e.button !== 0 || e.target.closest("button")) return;
  dragFrom = [window.screenX, window.screenY];
  // The acrylic is off while dragging (actions.rs actions_drag_start); a solid fill stands in for it.
  $("panel").classList.add("dragging");
  invoke("actions_drag_start");
  win.startDragging().catch(console.error);
});
listen("actions-drag-end", () => $("panel").classList.remove("dragging"));
document.addEventListener("click", (e) => {
  const moved = dragFrom && (window.screenX !== dragFrom[0] || window.screenY !== dragFrom[1]);
  dragFrom = null;
  if (moved) {
    fill?.cancel();
    $("tr-bar").style.visibility = "hidden";
    invoke("actions_keep");
    return;
  }
  if (!e.target.closest(".ac-menu")) invoke("actions_close");
});
$("ac-close").onclick = () => invoke("actions_close");
listen("actions-update", render);
render();
