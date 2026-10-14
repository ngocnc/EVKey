const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
// Theme chosen in the settings window (same origin, so same local storage); "system" follows Windows.
const applyTheme = () => {
  const t = localStorage.getItem("evkey-theme") ?? "system";
  window.__TAURI__.window.getCurrentWindow().setTheme(t === "system" ? null : t).catch(console.error);
};
applyTheme();
window.addEventListener("storage", (e) => e.key === "evkey-theme" && applyTheme());
const $ = (id) => document.getElementById(id);
const INPUTS = { telex: "Telex", vni: "VNI", simple_telex1: "Simple Telex 1", simple_telex2: "Simple Telex 2" };
// Table labels are read per build, since the UI language can change while the popup lives.
const tables = () => ({ unicode: "Unicode", tcvn3: "TCVN3 (ABC)", vni_windows: "VNI Windows", unicode_compound: t("table.unicode_compound") });
let builtLang = null;

// Runs a command, then re-reads the state so the popup reflects what the backend stored.
async function act(command, args) {
  try {
    await invoke(command, args);
  } catch (e) {
    console.error(e);
  }
  await refresh();
}

// Resizes the window to the panel, which fills it edge to edge; the Rust side keeps it anchored to the tray.
function fit() {
  requestAnimationFrame(() => invoke("popup_resize", { height: $("panel").offsetHeight }).catch(console.error));
}

function buildChoices(listId, options, command, arg) {
  const list = $(listId);
  for (const [value, label] of Object.entries(options)) {
    const b = Object.assign(document.createElement("button"), { className: "choice" });
    b.dataset.value = value;
    b.append(label, Object.assign(document.createElement("span"), { className: "check", textContent: "✓" }));
    b.onclick = () => act(command, { [arg]: value });
    list.append(b);
  }
}

function mark(listId, value) {
  for (const b of $(listId).children) b.classList.toggle("on", b.dataset.value === value);
}

// Rebuilds both choice lists in the given UI language; runs only when the language changes.
function buildLists(lang) {
  setUiLanguage(lang);
  applyI18n();
  $("input-list").replaceChildren();
  $("table-list").replaceChildren();
  buildChoices("input-list", INPUTS, "popup_set_input", "kind");
  buildChoices("table-list", tables(), "popup_set_table", "table");
  builtLang = lang;
}

async function refresh() {
  try {
    const s = await invoke("popup_state");
    if (s.ui_language !== builtLang) buildLists(s.ui_language);
    $("mode").checked = s.mode === "vietnamese";
    $("input-value").textContent = INPUTS[s.input_type];
    $("table-value").textContent = tables()[s.code_table];
    mark("input-list", s.input_type);
    mark("table-list", s.code_table);
    fit();
  } catch (e) {
    console.error(e);
  }
}

for (const name of ["input", "table"]) {
  $(name + "-head").onclick = () => {
    const section = $(name + "-section");
    section.classList.toggle("open");
    $(name + "-list").hidden = !section.classList.contains("open");
    fit();
  };
}

$("mode").onchange = () => act("popup_toggle_mode");
$("macro").onclick = () => act("popup_open_settings", { tab: "macro" });
$("settings").onclick = () => act("popup_open_settings", { tab: null });
$("quit").onclick = () => act("quit");

document.addEventListener("keydown", (e) => {
  if (e.key === "Escape") invoke("popup_hide");
});

listen("popup-show", refresh);
window.addEventListener("focus", refresh);
applyI18n();
refresh();
