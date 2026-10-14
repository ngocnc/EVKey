const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const $ = (id) => document.getElementById(id);

const applyTheme = () => {
  const th = localStorage.getItem("evkey-theme") ?? "system";
  window.__TAURI__.window.getCurrentWindow().setTheme(th === "system" ? null : th).catch(console.error);
};
applyTheme();

// Menu entries in display order: action id (as actions.rs expects) and its i18n key.
const MENU = [
  ["translate", "actions.translate"],
  ["add_diacritics", "actions.add_diacritics"],
  ["fix_spelling", "actions.fix_spelling"],
  ["rewrite_formal", "actions.rewrite_formal"],
  ["rewrite_shorter", "actions.rewrite_shorter"],
  ["rewrite_friendly", "actions.rewrite_friendly"],
  ["convert", "actions.convert"],
];

const menu = $("ac-menu");
for (const [id, key] of MENU) {
  const btn = document.createElement("button");
  btn.type = "button";
  btn.className = "ac-item";
  btn.dataset.key = key;
  btn.dataset.ai = String(!["translate", "convert"].includes(id));
  btn.onclick = (e) => {
    // A menu click runs an action; it must not close the panel.
    e.stopPropagation();
    invoke("actions_run", { action: id });
  };
  menu.append(btn);
}
// Shown when no Gemini key is saved: opens the Translate tab where the key is entered.
const setup = Object.assign(document.createElement("button"), { type: "button", className: "ac-setup" });
setup.dataset.key = "actions.setup_gemini";
setup.onclick = () => invoke("popup_open_settings", { tab: "translate" });
menu.append(setup);

let fill = null;

async function render() {
  const v = await invoke("actions_state");
  setUiLanguage(v.ui_language);
  const menuOpen = v.status === "menu";
  const loading = v.status === "loading";
  for (const btn of menu.children) {
    btn.textContent = t(btn.dataset.key);
    if (btn.dataset.ai === "true") btn.disabled = !v.gemini;
  }
  setup.hidden = v.gemini;
  menu.hidden = !menuOpen;
  const text = $("tr-text");
  text.hidden = menuOpen;
  text.textContent = loading ? t("translate.loading") : v.text;
  text.className = "tr-text " + (loading ? "loading" : v.status === "error" ? "error" : "");
  // The countdown bar starts when the panel is shown and runs for the configured time.
  fill?.cancel();
  // Hidden while loading and when the panel never closes by itself; it keeps its space so the content does not jump.
  const timed = !loading && !menuOpen && v.ms > 0;
  $("tr-bar").style.visibility = timed ? "visible" : "hidden";
  if (timed) fill = $("tr-fill").animate([{ transform: "scaleX(1)" }, { transform: "scaleX(0)" }], { duration: v.ms, easing: "linear", fill: "forwards" });
  requestAnimationFrame(() => invoke("actions_fit", { height: Math.ceil($("panel").getBoundingClientRect().height) }));
}

// The panel never takes focus, so there is no Esc. The x button closes it; a plain click on it does too.
// Dragging moves it and keeps it open (the auto-hide timer and countdown stop).
const win = window.__TAURI__.window.getCurrentWindow();
let dragFrom = null;
$("panel").addEventListener("mousedown", (e) => {
  if (e.button !== 0 || e.target.closest("button")) return;
  dragFrom = [window.screenX, window.screenY];
  win.startDragging().catch(console.error);
});
document.addEventListener("click", (e) => {
  const moved = dragFrom && (window.screenX !== dragFrom[0] || window.screenY !== dragFrom[1]);
  dragFrom = null;
  if (moved) {
    fill?.cancel();
    $("tr-bar").style.visibility = "hidden";
    invoke("actions_keep");
    return;
  }
  if (!e.target.closest(".ac-item, .ac-setup")) invoke("actions_close");
});
$("ac-close").onclick = () => invoke("actions_close");
listen("actions-update", render);
render();
