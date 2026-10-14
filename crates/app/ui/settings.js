const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const $ = (id) => document.getElementById(id);
const BOOLS = [
  "free_mark", "remember_app_mode", "undo_word", "check_spelling", "restore_if_wrong_spelling", "quick_telex",
  "quick_start_consonant", "quick_end_consonant", "upper_case_first_char", "upper_case_after_punct",
  "upper_case_after_enter", "upper_case_first_letter",
  "allow_consonant_zfwj", "auto_caps_macro", "toggle_sound", "auto_check_updates",
];
// Segments bound to a config field; `data-local` ones are look-only preferences kept in local storage.
const SEGS = [...document.querySelectorAll(".seg:not([data-local])")];
const ALL_SEGS = [...document.querySelectorAll(".seg")];
let state = null;
let toastTimer = 0;

function say(text, cls = "") {
  const el = $("msg");
  el.textContent = text;
  el.className = "toast show " + cls;
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => (el.className = "toast"), 2600);
}

async function run(action, okText) {
  try {
    await action();
    say(okText, "ok");
    return true;
  } catch (e) {
    say(String(e), "err");
    return false;
  }
}

function lines(text) {
  return text.split("\n").map((x) => x.trim()).filter(Boolean);
}

// One macro row. `saved` is the abbreviation as stored on disk (absent for a row that was never saved).
// Typing into the last row appends a fresh empty one, so there is always a row to type in and no "add" button.
function addMacroRow(key = "", value = "", saved) {
  const row = document.createElement("div");
  row.className = "macro-row";
  if (saved !== undefined) row.dataset.saved = saved;
  const del = Object.assign(document.createElement("button"), { className: "ghost", textContent: t("common.delete") });
  del.onclick = () => deleteMacroRow(row);
  const mk = Object.assign(document.createElement("input"), { className: "mk", placeholder: t("macro.key_placeholder"), value: key });
  const mv = Object.assign(document.createElement("input"), { className: "mv", placeholder: t("macro.value_placeholder"), value });
  for (const input of [mk, mv]) {
    input.oninput = () => {
      input.classList.remove("invalid");
      if (row === $("macro-rows").lastElementChild && (mk.value.trim() || mv.value.trim())) addMacroRow();
    };
  }
  row.append(mk, mv, del);
  $("macro-rows").append(row);
}

// Keeps one empty row at the end.
function ensureBlankMacroRow() {
  const last = $("macro-rows").lastElementChild;
  if (!last || last.querySelector(".mk").value.trim() || last.querySelector(".mv").value.trim()) addMacroRow();
}

// Deleting takes effect at once: a stored macro is removed from disk right away, without pressing Save.
async function deleteMacroRow(row) {
  const saved = row.dataset.saved;
  row.remove();
  ensureBlankMacroRow();
  if (saved === undefined) return;
  const rows = Object.entries(state.macros).filter(([k]) => k !== saved).map(([key, value]) => ({ key, value }));
  if (await run(() => invoke("save_macros", { rows }), t("toast.macro_deleted"))) delete state.macros[saved];
  else await load();
}

function renderCustomConsonants(list) {
  $("custom-consonants").replaceChildren(
    ...list.map((k) => {
      const chip = Object.assign(document.createElement("span"), { className: "chip" });
      const del = Object.assign(document.createElement("button"), { className: "chip-x", textContent: "×", title: t("common.delete") });
      del.onclick = () => setCustomConsonants(state.settings.custom_consonants.filter((x) => x !== k));
      chip.append(Object.assign(document.createElement("b"), { textContent: k.toUpperCase() }), del);
      return chip;
    }),
  );
}

// Every add/remove saves immediately, like the other settings.
async function setCustomConsonants(list) {
  const next = { ...readForm(), custom_consonants: list };
  try {
    await invoke("save_settings", { settings: next });
    state.settings = next;
    renderCustomConsonants(list);
  } catch (e) {
    say(String(e), "err");
  }
}

// The error shows under the field, where the user is looking, not only in the toast.
function consonantError(text) {
  $("custom-consonant-error").textContent = text;
  $("custom-consonant-error").hidden = !text;
}

async function addCustomConsonant() {
  const input = $("custom-consonant-input");
  const k = input.value.trim().toLowerCase();
  if (!k) return consonantError("");
  if (!/^[a-z]$/.test(k)) return consonantError(t("custom.invalid"));
  if (state.settings.custom_consonants.includes(k)) return consonantError(t("custom.duplicate", { key: k }));
  consonantError("");
  input.value = "";
  await setCustomConsonants([...state.settings.custom_consonants, k]);
}

// Option description: under a segmented control, the text of the option that is active (strings "hint.<seg>.<value>").
const segId = (seg) => seg.dataset.key ?? "local-" + seg.dataset.local;
function paintHint(seg, value) {
  const key = `hint.${segId(seg)}.${value}`;
  const text = t(key);
  let hint = seg.nextElementSibling?.classList.contains("seg-hint") ? seg.nextElementSibling : null;
  if (text === key) return hint?.remove();
  if (!hint) seg.after((hint = Object.assign(document.createElement("p"), { className: "seg-hint" })));
  hint.textContent = text;
}
const activeValue = (seg) => seg.querySelector("button.active")?.dataset.value;

function setSeg(seg, value) {
  // A hotkey written by hand in config.toml that is not one of the presets gets its own button, so saving never rewrites it.
  if (seg.dataset.key === "toggle_hotkey") {
    seg.querySelector(".extra")?.remove();
    if (![...seg.children].some((b) => b.dataset.value === value)) {
      seg.append(Object.assign(document.createElement("button"), { className: "extra", textContent: value === "none" ? t("common.off") : value }));
      seg.lastChild.dataset.value = value;
    }
  }
  for (const b of seg.children) b.classList.toggle("active", b.dataset.value === value);
  paintHint(seg, value);
  placeThumb(seg);
}



// Secure Input banner look: the preview uses the same renderer as the overlay window.
const BANNER_FIELDS = ["bn-opacity", "bn-blur", "bn-bg", "bn-border", "bn-text"];

function renderBannerPreview() {
  $("bn-opacity-value").textContent = $("bn-opacity").value + "%";
  $("bn-blur-value").textContent = $("bn-blur").value + "%";
  renderBanner($("bn-preview"), {
    app: "Chrome",
    opacity: Number($("bn-opacity").value),
    blur: Number($("bn-blur").value),
    bg: $("bn-bg").value,
    border: $("bn-border").value,
    text: $("bn-text").value,
  });
}

function fillBanner(s) {
  $("bn-opacity").value = s.banner_opacity;
  $("bn-blur").value = s.banner_blur;
  $("bn-bg").value = s.banner_bg;
  $("bn-border").value = s.banner_border;
  $("bn-text").value = s.banner_text;
  renderBannerPreview();
}
function fill() {
  const s = state.settings;
  setUiLanguage(s.ui_language);
  applyI18n();
  for (const k of BOOLS) $(k).checked = s[k];
  for (const seg of SEGS) setSeg(seg, s[seg.dataset.key]);
  $("excluded_apps").value = s.excluded_apps.join("\n");
  $("macro-rows").replaceChildren();
  for (const [k, v] of Object.entries(state.macros)) addMacroRow(k, v, k);
  addMacroRow();
  renderCustomConsonants(s.custom_consonants);
  $("autostart").checked = state.autostart;
  $("config-dir").textContent = state.config_dir;
  $("version").textContent = "v" + state.version;
  $("about-version").textContent = "v" + state.version;
  $("config-error").hidden = !state.config_error;
  $("config-error-text").textContent = state.config_error ?? "";
  $("excluded_apps").placeholder = "game.exe";
  fillBanner(s);
  fillTime(s.translate_popup_ms);
  paintHotkey();
  paintWelcome();
  paintGemini();
  updateVisibility();
  syncRanges();
}

// Switches the UI language without rebuilding inputs, so unsaved macro edits stay.
function applyLanguage(lang) {
  setUiLanguage(lang);
  applyI18n();
  renderBannerPreview();
  for (const seg of document.querySelectorAll(".seg")) if (activeValue(seg)) paintHint(seg, activeValue(seg));
  renderCustomConsonants(state.settings.custom_consonants);
  for (const row of document.querySelectorAll(".macro-row")) {
    row.querySelector(".mk").placeholder = t("macro.key_placeholder");
    row.querySelector(".mv").placeholder = t("macro.value_placeholder");
    row.querySelector(".ghost").textContent = t("common.delete");
  }
  $("pin").title = $("pin").classList.contains("active") ? t("pin.unpin") : t("pin.pin");
  paintHotkey();
  paintWelcome();
  paintGemini();
  $("translate-unit").refresh();
}

function readForm() {
  const s = { ...state.settings };
  for (const k of BOOLS) s[k] = $(k).checked;
  for (const seg of SEGS) s[seg.dataset.key] = seg.querySelector(".active")?.dataset.value ?? s[seg.dataset.key];
  s.excluded_apps = lines($("excluded_apps").value);
  s.banner_opacity = Number($("bn-opacity").value);
  s.banner_blur = Number($("bn-blur").value);
  s.banner_bg = $("bn-bg").value;
  s.banner_border = $("bn-border").value;
  s.banner_text = $("bn-text").value;
  s.translate_popup_ms = timeMs();
  return s;
}

async function load() {
  try {
    state = await invoke("get_state");
    fill();
  } catch (e) {
    say(String(e), "err");
  }
}

// Settings save as soon as they change; the engine picks the file up within a second.
async function saveSettings() {
  try {
    await invoke("save_settings", { settings: readForm() });
    state.settings = readForm();
  } catch (e) {
    say(String(e), "err");
  }
}

// A row needs both the abbreviation and its text; a completely empty row is ignored, a half-filled one blocks saving.
async function saveMacros() {
  const rows = [];
  let firstBad = 0;
  for (const [i, r] of [...document.querySelectorAll(".macro-row")].entries()) {
    const [mk, mv] = [r.querySelector(".mk"), r.querySelector(".mv")];
    const [hasKey, hasValue] = [!!mk.value.trim(), !!mv.value.trim()];
    if (hasKey === hasValue) {
      if (hasKey) rows.push({ key: mk.value, value: mv.value });
      continue;
    }
    (hasKey ? mv : mk).classList.add("invalid");
    firstBad ||= i + 1;
  }
  if (firstBad) {
    document.querySelectorAll(".macro-row input.invalid")[0].focus();
    return say(t("macro.row_incomplete", { n: firstBad }), "err");
  }
  if (await run(() => invoke("save_macros", { rows }), t("toast.macros_saved"))) await load();
}

async function resetConfig() {
  if (!confirm(t("reset.confirm"))) return;
  if (await run(() => invoke("reset_config"), t("toast.reset_ok"))) await load();
}

async function addLastApp() {
  try {
    const app = await invoke("last_app");
    if (!app) return say(t("apps.none_recorded"));
    const list = lines($("excluded_apps").value);
    if (!list.some((x) => x.toLowerCase() === app.toLowerCase())) list.push(app);
    $("excluded_apps").value = list.join("\n");
    await saveSettings();
  } catch (e) {
    say(String(e), "err");
  }
}

let currentTab = 0;
const tabs = [...document.querySelectorAll(".tab")];
function showTab(name) {
  const index = tabs.findIndex((t) => t.dataset.tab === name);
  if (index < 0 || index === currentTab) return;
  tabs.forEach((t, i) => t.classList.toggle("active", i === index));
  for (const p of document.querySelectorAll(".pane")) {
    p.hidden = p.dataset.pane !== name;
    p.classList.toggle("from-left", index < currentTab);
  }
  currentTab = index;
}
for (const tab of tabs) tab.onclick = () => showTab(tab.dataset.tab);
listen("settings-tab", (e) => showTab(e.payload));
if (window.__evkeyTab) showTab(window.__evkeyTab);

// Liquid glass look: look-only preferences in local storage (contract in glass.js); not saved to config.toml.
const glassStrength = $("glass-strength");
const glassTint = $("glass-tint");
const glassTintEnd = $("glass-tint-end");
const glassGradient = $("glass-gradient");
const defaultTint = matchMedia("(prefers-color-scheme: dark)").matches ? "#000000" : "#ffffff";
function syncGlassControls() {
  glassStrength.value = glassGet("strength");
  $("glass-strength-value").textContent = glassStrength.value + "%";
  glassTint.value = glassGet("tint") || defaultTint;
  glassTintEnd.value = glassGet("tint-end") || companionTint(glassTint.value);
  glassGradient.checked = glassGet("tint-end") !== "";
  $("glass-tint-end-wrap").hidden = !glassGradient.checked;
  $("accent-color").value = glassGet("accent") || "#ff6b8a";
  syncRanges();
}
syncGlassControls();
$("accent-color").oninput = () => {
  localStorage.setItem("evkey-glass-accent", $("accent-color").value);
  applyGlassLook();
};
$("accent-default").onclick = () => {
  localStorage.removeItem("evkey-glass-accent");
  $("accent-color").value = "#ff6b8a";
  applyGlassLook();
};
glassStrength.oninput = () => {
  localStorage.setItem("evkey-glass-strength", glassStrength.value);
  $("glass-strength-value").textContent = glassStrength.value + "%";
  applyGlassLook();
};
glassTint.oninput = () => {
  localStorage.setItem("evkey-glass-tint", glassTint.value);
  applyGlassLook();
};
glassGradient.onchange = () => {
  const on = glassGradient.checked;
  glassTintEnd.value = companionTint(glassTint.value);
  localStorage.setItem("evkey-glass-tint-end", on ? glassTintEnd.value : "");
  $("glass-tint-end-wrap").hidden = !on;
  applyGlassLook();
};
glassTintEnd.oninput = () => {
  localStorage.setItem("evkey-glass-tint-end", glassTintEnd.value);
  applyGlassLook();
};
$("gl-reset").onclick = async () => {
  for (const k of Object.keys(localStorage)) if (k.startsWith("evkey-glass-")) localStorage.removeItem(k);
  localStorage.removeItem("evkey-theme");
  syncGlassControls();
  for (const seg of localSegs) {
    paintLocalSeg(seg, LOCAL_SEG_DEFAULTS[seg.dataset.local]);
    placeThumb(seg);
  }
  applyTheme("system");
  applyGlassLook();
  const d = await invoke("banner_defaults");
  $("bn-opacity").value = d.opacity;
  $("bn-blur").value = d.blur;
  $("bn-bg").value = d.bg;
  $("bn-border").value = d.border;
  $("bn-text").value = d.text;
  renderBannerPreview();
  syncRanges();
  saveSettings();
};
for (const seg of SEGS) {
  seg.onclick = (e) => {
    const b = e.target.closest("button");
    if (!b) return;
    for (const x of seg.children) x.classList.toggle("active", x === b);
    paintHint(seg, b.dataset.value);
    // The welcome screen repeats the input-method control; keep every copy of a key in step before saving.
    for (const other of SEGS) if (other !== seg && other.dataset.key === seg.dataset.key) setSeg(other, b.dataset.value);
    saveSettings();
    if (seg.dataset.key === "ui_language") applyLanguage(b.dataset.value);
    if (seg.dataset.key === "input_type") paintWelcome();
  };
}
// Look-only segmented controls (theme, glass style) persist in local storage; the popup reads the same keys.
const LOCAL_SEG_DEFAULTS = { theme: "system", "glass-style": "regular" };
const localSegs = [...document.querySelectorAll(".seg[data-local]")];
const localKey = (seg) => "evkey-" + seg.dataset.local;
function paintLocalSeg(seg, value) {
  for (const b of seg.querySelectorAll("button")) b.classList.toggle("active", b.dataset.value === value);
  paintHint(seg, value);
}
// Colour theme of the settings and popup windows; "system" follows Windows.
function applyTheme(theme) {
  window.__TAURI__.window.getCurrentWindow().setTheme(theme === "system" ? null : theme).catch((e) => say(String(e), "err"));
}
for (const seg of localSegs) paintLocalSeg(seg, localStorage.getItem(localKey(seg)) ?? LOCAL_SEG_DEFAULTS[seg.dataset.local]);
applyTheme(localStorage.getItem("evkey-theme") ?? "system");
for (const seg of localSegs) {
  seg.onclick = (e) => {
    const b = e.target.closest("button");
    if (!b) return;
    paintLocalSeg(seg, b.dataset.value);
    localStorage.setItem(localKey(seg), b.dataset.value);
    if (seg.dataset.local === "theme") applyTheme(b.dataset.value);
    else applyGlassLook();
  };
}
for (const k of BOOLS) $(k).onchange = saveSettings;
$("excluded_apps").onchange = saveSettings;
$("save-macros").onclick = saveMacros;
$("add-last-app").onclick = addLastApp;
$("add-custom-consonant").onclick = addCustomConsonant;
// A DOM0 handler that returns false cancels the key, so this must not return the `&&` result.
$("custom-consonant-input").onkeydown = (e) => {
  if (e.key === "Enter") addCustomConsonant();
};
// A letter is added as soon as it is typed; the + button and Enter stay as a way to retry.
$("custom-consonant-input").oninput = addCustomConsonant;
const appWindow = window.__TAURI__.window.getCurrentWindow();
// Settings window starts pinned; keep the real window level in sync with the pin button.
// The choice is remembered in local storage (pinned unless the user unpinned it).
const paintPin = (on) => {
  $("pin").classList.toggle("active", on);
  $("pin").setAttribute("aria-pressed", String(on));
  $("pin").title = on ? t("pin.unpin") : t("pin.pin");
};
const pinned = localStorage.getItem("evkey-pin") !== "0";
paintPin(pinned);
appWindow.setAlwaysOnTop(pinned).catch((e) => say(String(e), "err"));
$("pin").onclick = async () => {
  const pin = $("pin");
  const on = !pin.classList.contains("active");
  try {
    await appWindow.setAlwaysOnTop(on);
    localStorage.setItem("evkey-pin", on ? "1" : "0");
    paintPin(on);
  } catch (e) {
    say(String(e), "err");
  }
};
$("autostart").onchange = () =>
  run(() => invoke("set_autostart", { on: $("autostart").checked }), t("toast.autostart_ok"));
$("open-folder").onclick = () => run(() => invoke("open_config_dir"), t("toast.open_folder_ok"));
$("reset").onclick = resetConfig;
for (const id of BANNER_FIELDS) {
  $(id).oninput = () => renderBannerPreview();
  $(id).onchange = saveSettings;
}
$("reset-broken").onclick = resetConfig;

// Translate and action hotkeys: click a `.rec` button (`data-key` = settings field), then press the new combination (Esc cancels).
const HOTKEY_NAMES = { ctrl: "Ctrl", alt: "Alt", shift: "Shift", win: "Win", space: "Space" };
const HOTKEY_BTNS = [...document.querySelectorAll("button.rec[data-key]")];
let recordingHotkey = null;
function paintHotkey() {
  for (const btn of HOTKEY_BTNS) {
    const value = state?.settings[btn.dataset.key] ?? "none";
    btn.classList.toggle("recording", recordingHotkey === btn);
    btn.replaceChildren();
    if (recordingHotkey === btn) btn.textContent = t("translate.hotkey.recording");
    else if (value === "none") btn.textContent = t("translate.hotkey.unset");
    else for (const part of value.split("+")) btn.append(Object.assign(document.createElement("kbd"), { textContent: HOTKEY_NAMES[part] ?? part.toUpperCase() }));
    btn.nextElementSibling.disabled = value === "none";
  }
}
// Key name in the config grammar (letter, digit, f1-f12, space) for a keydown event, or null.
function hotkeyKey(e) {
  let m;
  if ((m = e.code.match(/^Key([A-Z])$/))) return m[1].toLowerCase();
  if ((m = e.code.match(/^Digit(\d)$/))) return m[1];
  if ((m = e.code.match(/^F(\d+)$/)) && m[1] >= 1 && m[1] <= 12) return "f" + Number(m[1]);
  return e.code === "Space" ? "space" : null;
}
async function setHotkey(btn, value) {
  recordingHotkey = null;
  state.settings[btn.dataset.key] = value;
  await saveSettings();
  paintHotkey();
}
for (const btn of HOTKEY_BTNS) {
  btn.onclick = () => {
    recordingHotkey = btn;
    paintHotkey();
  };
  btn.onblur = () => {
    recordingHotkey = null;
    paintHotkey();
  };
  btn.onkeydown = (e) => {
    if (recordingHotkey !== btn) return;
    e.preventDefault();
    if (["Control", "Shift", "Alt", "Meta"].includes(e.key)) return;
    if (e.key === "Escape") {
      recordingHotkey = null;
      paintHotkey();
      return;
    }
    const key = hotkeyKey(e);
    if (!key) return say(t("translate.hotkey.bad_key"), "err");
    const mods = [e.ctrlKey && "ctrl", e.altKey && "alt", e.shiftKey && "shift"].filter(Boolean);
    if (!mods.length && !/^f\d+$/.test(key)) return say(t("translate.hotkey.need_modifier"), "err");
    const value = [...mods, key].join("+");
    const other = HOTKEY_BTNS.find((b) => b !== btn && state.settings[b.dataset.key] === value);
    if (other) return say(t("hotkey.taken"), "err");
    setHotkey(btn, value);
  };
  btn.nextElementSibling.onclick = () => setHotkey(btn, "none");
}

// Popup display time: a number plus a unit (ms, s, or "never" = 0, the popup stays until clicked).
const TIME_MIN = { ms: 100, s: 1 };
function fillTime(ms) {
  const unit = ms === 0 ? "never" : ms % 1000 === 0 ? "s" : "ms";
  $("translate-unit").value = unit;
  $("translate-time").value = unit === "s" ? ms / 1000 : unit === "ms" ? ms : "";
  $("translate-time").disabled = unit === "never";
}
// The typed number for the current unit, kept within 100 ms .. 10 min.
function timeNumber() {
  const unit = $("translate-unit").value;
  const n = parseInt($("translate-time").value, 10) || 0;
  return Math.min(Math.max(n, TIME_MIN[unit]), unit === "s" ? 600 : 600000);
}
function timeMs() {
  const unit = $("translate-unit").value;
  return unit === "never" ? 0 : unit === "s" ? timeNumber() * 1000 : timeNumber();
}
// Custom dropdown (.dd): `el.value` gets/sets the chosen option, picking one fires "change" on el.
function initDropdown(el) {
  const btn = el.querySelector(".dd-btn");
  const menu = el.querySelector(".dd-menu");
  const items = [...menu.children];
  // Moved to <body>: a parent with backdrop-filter would make `position: fixed` relative to itself and clip the menu.
  document.body.append(menu);
  let focused = -1;
  const label = () => (btn.firstElementChild.textContent = items.find((li) => li.dataset.value === el.dataset.value)?.textContent ?? "");
  const mark = (i) => items.forEach((li, n) => li.classList.toggle("focus", n === i));
  const close = () => {
    menu.hidden = true;
    el.classList.remove("open");
    btn.setAttribute("aria-expanded", "false");
    document.removeEventListener("pointerdown", outside, true);
  };
  const outside = (e) => el.contains(e.target) || menu.contains(e.target) || close();
  const open = () => {
    for (const li of items) li.setAttribute("aria-selected", String(li.dataset.value === el.dataset.value));
    menu.hidden = false;
    // Below the button, right edges aligned; above it when there is no room underneath.
    const r = btn.getBoundingClientRect();
    menu.style.minWidth = r.width + "px";
    menu.style.left = "0px"; // measure at its natural width, not squeezed by the auto position
    const h = menu.offsetHeight;
    menu.style.top = (r.bottom + 6 + h > innerHeight ? r.top - 6 - h : r.bottom + 6) + "px";
    menu.style.left = Math.max(8, r.right - menu.offsetWidth) + "px";
    el.classList.add("open");
    btn.setAttribute("aria-expanded", "true");
    focused = items.findIndex((li) => li.dataset.value === el.dataset.value);
    mark(focused);
    document.addEventListener("pointerdown", outside, true);
  };
  const pick = (li) => {
    close();
    btn.focus();
    if (li.dataset.value === el.dataset.value) return;
    el.dataset.value = li.dataset.value;
    label();
    el.dispatchEvent(new Event("change"));
  };
  Object.defineProperty(el, "value", {
    get: () => el.dataset.value,
    set: (v) => {
      el.dataset.value = v;
      label();
    },
  });
  el.refresh = label;
  btn.onclick = () => (menu.hidden ? open() : close());
  btn.onkeydown = (e) => {
    if (menu.hidden) {
      if (["ArrowDown", "ArrowUp"].includes(e.key)) {
        e.preventDefault();
        open();
      }
      return;
    }
    if (e.key === "ArrowDown" || e.key === "ArrowUp") {
      e.preventDefault();
      focused = (focused + (e.key === "ArrowDown" ? 1 : -1) + items.length) % items.length;
      mark(focused);
    } else if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      if (items[focused]) pick(items[focused]);
    } else if (e.key === "Escape") {
      e.preventDefault();
      close();
    } else if (e.key === "Tab") close();
  };
  menu.onclick = (e) => {
    const li = e.target.closest("li");
    if (li) pick(li);
  };
  window.addEventListener("blur", close);
  label();
}
initDropdown($("translate-unit"));

$("translate-time").onchange = () => {
  $("translate-time").value = timeNumber();
  saveSettings();
};
$("translate-unit").onchange = () => {
  const unit = $("translate-unit").value;
  const ms = state.settings.translate_popup_ms || 5000;
  $("translate-time").disabled = unit === "never";
  $("translate-time").value = unit === "s" ? Math.max(1, Math.round(ms / 1000)) : unit === "ms" ? ms : "";
  saveSettings();
};
$("check-update").onclick = async () => {
  const btn = $("check-update");
  $("update-status").textContent = "";
  btn.disabled = true;
  btn.classList.add("loading");
  btn.textContent = t("about.update.checking");
  let r;
  try {
    r = await invoke("check_update_now");
  } finally {
    btn.disabled = false;
    btn.classList.remove("loading");
    btn.textContent = t("about.update.check");
  }
  if (r.status === "latest") $("update-status").textContent = t("about.update.latest");
  if (r.status === "error") $("update-status").textContent = t("about.update.error", { error: r.error });
};
$("open-github").onclick = () => run(() => invoke("open_github"), t("toast.open_github_ok"));
listen("settings-show", load);
applyI18n();
load().then(() => {
  if (!localStorage.getItem("evkey-welcomed")) showWelcome();
});

// Advanced sections: closed by default, the open state is remembered per pane.
for (const d of document.querySelectorAll("details.advanced")) {
  const key = "evkey-adv-" + d.dataset.adv;
  d.open = localStorage.getItem(key) === "1";
  d.ontoggle = () => {
    localStorage.setItem(key, d.open ? "1" : "0");
    updateVisibility();
  };
}

// Welcome screen: first open only. Its input-method control is a second `input_type` segment (see the SEGS handler).
function paintWelcome() {
  const hk = state?.settings.toggle_hotkey ?? "none";
  const line = $("welcome-hotkey");
  if (hk === "none") line.textContent = t("welcome.toggle_none");
  else {
    const [before, after] = t("welcome.toggle_line", { keys: "\u0000" }).split("\u0000");
    const keys = hk.split("+").map((p) => Object.assign(document.createElement("kbd"), { textContent: HOTKEY_NAMES[p] ?? p.toUpperCase() }));
    line.replaceChildren(before, ...keys, after ?? "");
  }
  $("welcome-try").placeholder = t(state?.settings.input_type === "vni" ? "welcome.try.vni" : "welcome.try.telex");
}
function showWelcome() {
  $("welcome").hidden = false;
  $("welcome-try").focus();
}
function closeWelcome() {
  localStorage.setItem("evkey-welcomed", "1");
  $("welcome").hidden = true;
}
$("welcome-start").onclick = closeWelcome;
$("welcome-skip").onclick = closeWelcome;
$("welcome").addEventListener("keydown", (e) => {
  if (e.key === "Escape") closeWelcome();
});

// Picks a file with the native file chooser and resolves with its File (null when cancelled).
function pickFile(input) {
  return new Promise((resolve) => {
    input.value = "";
    input.onchange = () => resolve(input.files[0] ?? null);
    input.oncancel = () => resolve(null);
    input.click();
  });
}
$("macro-import").onclick = async () => {
  const file = await pickFile($("macro-import-file"));
  if (!file) return;
  const replace = $("macro-import-replace").checked;
  if (replace && !confirm(t("macro.import.confirm"))) return;
  try {
    const content = [...new Uint8Array(await file.arrayBuffer())];
    const n = await invoke("import_macros", { content, replace });
    say(t("macro.import.done", { n }), "ok");
    await load();
  } catch (e) {
    say(String(e), "err");
  }
};
$("config-export").onclick = async () => {
  try {
    const path = await invoke("export_config");
    say(t("system.export.done", { path }), "ok");
  } catch (e) {
    say(String(e), "err");
  }
};
$("config-import").onclick = async () => {
  const file = await pickFile($("config-import-file"));
  if (!file || !confirm(t("system.import.confirm"))) return;
  if (await run(async () => invoke("import_config", { content: await file.text() }), t("system.import.done"))) await load();
};

// Gemini API key: the page only learns whether a key exists; the key itself never comes back from the app.
let geminiSet = false;
function paintGemini() {
  $("gemini-key").placeholder = t(geminiSet ? "gemini.key.placeholder_set" : "gemini.key.placeholder_unset");
  $("gemini-status").textContent = t(geminiSet ? "gemini.key.status_set" : "gemini.key.status_unset");
  $("gemini-delete").disabled = !geminiSet;
}
async function refreshGemini() {
  geminiSet = await invoke("gemini_key_status").catch(() => false);
  paintGemini();
}
$("gemini-save").onclick = async () => {
  const key = $("gemini-key").value.trim();
  if (!key) return $("gemini-key").focus();
  const btn = $("gemini-save");
  btn.disabled = true;
  btn.classList.add("loading");
  $("gemini-status").textContent = t("gemini.key.checking");
  try {
    await invoke("gemini_key_set", { key });
    $("gemini-key").value = "";
    say(t("gemini.key.saved"), "ok");
  } catch (e) {
    say(String(e), "err");
  } finally {
    btn.disabled = false;
    btn.classList.remove("loading");
    await refreshGemini();
  }
};
$("gemini-key").onkeydown = (e) => {
  if (e.key === "Enter") $("gemini-save").click();
};
$("gemini-delete").onclick = async () => {
  if (await run(() => invoke("gemini_key_delete"), t("gemini.key.deleted"))) await refreshGemini();
};
refreshGemini();

// Items marked data-when ("key", "key=value" or "key!=value") only show while that option is chosen.
function currentValue(key) {
  const seg = document.querySelector(`.seg[data-key="${key}"]`);
  if (seg) return seg.querySelector(".active")?.dataset.value;
  return $(key)?.checked;
}

const reduceMotion = matchMedia("(prefers-reduced-motion: reduce)");
// Animates a data-when item in or out; the first pass in updateVisibility sets it without animation.
function animateWhen(el, show) {
  el._anim?.cancel();
  el.style.overflow = "";
  if (reduceMotion.matches) {
    el.hidden = !show;
    return;
  }
  if (show) el.hidden = false;
  const cs = getComputedStyle(el);
  const full = { height: cs.height, paddingTop: cs.paddingTop, paddingBottom: cs.paddingBottom, opacity: "1" };
  const zero = { height: "0px", paddingTop: "0px", paddingBottom: "0px", opacity: "0" };
  const [from, to] = show ? [zero, full] : [full, zero];
  el.style.overflow = "hidden";
  el._anim = el.animate([from, to], { duration: 260, easing: "cubic-bezier(0.22,1,0.36,1)" });
  el._anim.onfinish = () => {
    el.hidden = !show;
    el.style.overflow = "";
  };
}

function updateVisibility() {
  for (const el of document.querySelectorAll("[data-platform], [data-when]")) {
    let hide = false;
    if (el.dataset.when) {
      const [, key, op, want] = el.dataset.when.match(/^(\w+)(?:(!?=)(.+))?$/);
      const value = currentValue(key);
      hide ||= op === "=" ? value !== want : op === "!=" ? value === want : !value;
    }
    const show = !hide;
    if (el.dataset.vis === undefined) el.hidden = hide;
    else if (el.dataset.vis !== String(show)) animateWhen(el, show);
    el.dataset.vis = String(show);
  }
  for (const group of document.querySelectorAll(".group")) {
    const visible = [...group.querySelectorAll(".item")].filter((i) => !i.hidden && i.dataset.vis !== "false");
    for (const item of group.querySelectorAll(".item")) item.classList.toggle("last", item === visible.at(-1));
  }
}
document.addEventListener("change", updateVisibility);
document.addEventListener("click", (e) => e.target.closest(".seg") && updateVisibility());

// Each segmented control gets one highlight element that slides between its buttons.
function placeThumb(seg) {
  const b = seg.querySelector("button.active");
  const thumb = seg.querySelector(".thumb");
  if (!b || !b.offsetWidth) return;
  thumb.style.width = b.offsetWidth + "px";
  thumb.style.height = b.offsetHeight + "px";
  thumb.style.transform = `translate(${b.offsetLeft}px, ${b.offsetTop}px)`;
  seg.classList.add("ready");
}
const thumbObserver = new ResizeObserver((entries) => entries.forEach((e) => placeThumb(e.target)));
for (const seg of ALL_SEGS) {
  seg.prepend(Object.assign(document.createElement("i"), { className: "thumb" }));
  thumbObserver.observe(seg);
  seg.addEventListener("click", () => placeThumb(seg));
}

// Filled part of every range track (left of the thumb), via --p. Called after code sets values and on every input.
function syncRanges() {
  for (const r of document.querySelectorAll('input[type="range"]')) r.style.setProperty("--p", ((r.value - r.min) / (r.max - r.min)) * 100 + "%");
}
document.addEventListener("input", syncRanges);
syncRanges();
