const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const $ = (id) => document.getElementById(id);
const BOOLS = [
  "free_mark", "remember_app_mode", "undo_word", "check_spelling", "restore_if_wrong_spelling", "restore_english", "undo_english_restore", "quick_telex",
  "quick_start_consonant", "quick_end_consonant", "upper_case_first_char", "upper_case_after_punct",
  "upper_case_after_enter", "upper_case_first_letter",
  "allow_consonant_zfwj", "auto_caps_macro", "toggle_sound", "auto_check_updates",
];
// Space- or comma-separated word lists of restore_english.
const WORD_LISTS = ["english_words_extra", "english_words_excluded"];
// Segments bound to a config field; `data-local` ones are look-only preferences kept in local storage.
const SEGS = [...document.querySelectorAll(".seg:not([data-local])")];
const ALL_SEGS = [...document.querySelectorAll(".seg")];
const DDS = [...document.querySelectorAll(".dd[data-key]")];
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
      const del = Object.assign(document.createElement("button"), { className: "chip-x", title: t("common.delete") });
      del.setAttribute("aria-label", t("common.delete"));
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
  // A dropdown sits in a row with its label, so its hint goes under that row.
  const anchor = seg.closest(".dd-row") ?? seg;
  let hint = anchor.nextElementSibling?.classList.contains("seg-hint") ? anchor.nextElementSibling : null;
  if (text === key) return hint?.remove();
  if (!hint) anchor.after((hint = Object.assign(document.createElement("p"), { className: "seg-hint" })));
  hint.textContent = text;
}
const activeValue = (seg) => seg.querySelector("button.active")?.dataset.value;

function setSeg(seg, value) {
  for (const b of seg.children) b.classList.toggle("active", b.dataset.value === value);
  paintHint(seg, value);
  placeThumb(seg);
}



function fill() {
  const s = state.settings;
  setUiLanguage(s.ui_language);
  applyI18n();
  for (const k of BOOLS) $(k).checked = s[k];
  for (const seg of SEGS) setSeg(seg, s[seg.dataset.key]);
  for (const dd of DDS) setDropdown(dd, s[dd.dataset.key]);
  $("excluded_apps").value = s.excluded_apps.join("\n");
  for (const id of WORD_LISTS) $(id).value = s[id].join(" ");
  $("macro-rows").replaceChildren();
  for (const [k, v] of Object.entries(state.macros)) addMacroRow(k, v, k);
  addMacroRow();
  renderCustomConsonants(s.custom_consonants);
  // The elevated sign-in task also does the autostart, so the plain switch shows on and is locked while it exists.
  $("autostart").checked = state.autostart || state.admin;
  $("autostart").disabled = state.admin;
  $("admin").checked = state.admin;
  $("config-dir").textContent = state.config_dir;
  $("app-version").textContent = "v" + state.version;
  $("config-error").hidden = !state.config_error;
  $("config-error-text").textContent = state.config_error ?? "";
  $("excluded_apps").placeholder = "game.exe";
  paintHero();
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
  paintHero();
  for (const seg of document.querySelectorAll(".seg")) if (activeValue(seg)) paintHint(seg, activeValue(seg));
  for (const dd of DDS) setDropdown(dd, dd.value);
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
  for (const dd of DDS) s[dd.dataset.key] = dd.value || s[dd.dataset.key];
  s.excluded_apps = lines($("excluded_apps").value);
  for (const id of WORD_LISTS) s[id] = $(id).value.split(/[\s,]+/).filter(Boolean);
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
const navPill = document.querySelector(".nav-pill");
const reduceMotion = matchMedia("(prefers-reduced-motion: reduce)");
// Thanh chọn trượt nằm dưới tab đang mở (transform + height; chuyển động do CSS .nav-pill).
function placePill(index, instant = false) {
  const b = tabs[index];
  if (instant) navPill.style.transition = "none";
  navPill.style.height = b.offsetHeight + "px";
  navPill.style.transform = `translateY(${b.offsetTop}px)`;
  if (instant) {
    void navPill.offsetWidth;
    navPill.style.transition = "";
  }
}
function showTab(name) {
  const index = tabs.findIndex((t) => t.dataset.tab === name);
  if (index < 0 || index === currentTab) return;
  // Trang mới trượt vào từ phía tab được bấm: tab phía sau từ phải, tab phía trước từ trái.
  const dir = index > currentTab ? 1 : -1;
  tabs.forEach((t, i) => t.classList.toggle("active", i === index));
  placePill(index);
  const pane = document.querySelector(`.pane[data-pane="${name}"]`);
  for (const p of document.querySelectorAll(".pane")) p.hidden = p !== pane;
  if (!reduceMotion.matches) {
    pane.animate([{ opacity: 0, transform: `translateX(${dir * 28}px)` }, { opacity: 1, transform: "none" }], {
      duration: 300,
      easing: "cubic-bezier(.32,.72,0,1)",
    });
  }
  currentTab = index;
}
for (const tab of tabs) tab.onclick = () => showTab(tab.dataset.tab);
listen("settings-tab", (e) => showTab(e.payload));
if (window.__evkeyTab) showTab(window.__evkeyTab);

// Tùy chọn giao diện (glass.js giữ và áp lên <html>): sáng/tối, chất liệu, màu nhấn, độ trong suốt. Chỉ lưu trên máy này.
const transparencyInput = $("glass-transparency");
function paintSwatches() {
  const box = $("accent-swatches");
  if (!box.children.length) {
    box.replaceChildren(
      ...LOOK_ACCENTS.map((name) => {
        const b = Object.assign(document.createElement("button"), { type: "button", className: "swatch", title: name });
        b.dataset.accent = name;
        b.setAttribute("aria-label", name);
        b.onclick = () => {
          lookSet("accent", name);
          paintLook();
        };
        return b;
      }),
    );
  }
  for (const b of box.children) {
    const on = b.dataset.accent === lookGet("accent");
    b.classList.toggle("active", on);
    b.setAttribute("aria-pressed", String(on));
  }
}
function paintLook() {
  for (const seg of localSegs) paintLocalSeg(seg, lookGet(seg.dataset.local));
  paintSwatches();
  transparencyInput.value = lookGet("transparency");
  $("glass-transparency-value").textContent = transparencyInput.value + "%";
  syncRanges();
  updateVisibility();
}
transparencyInput.oninput = () => {
  lookSet("transparency", transparencyInput.value);
  $("glass-transparency-value").textContent = transparencyInput.value + "%";
};
$("gl-reset").onclick = () => {
  lookReset();
  paintLook();
};
for (const seg of SEGS) {
  seg.onclick = (e) => {
    const b = e.target.closest("button");
    if (!b) return;
    for (const x of seg.children) x.classList.toggle("active", x === b);
    paintHint(seg, b.dataset.value);
    // The welcome screen repeats the input-method control; keep every copy of a key in step before saving.
    for (const other of SEGS) if (other !== seg && other.dataset.key === seg.dataset.key) setSeg(other, b.dataset.value);
    for (const dd of DDS) if (dd.dataset.key === seg.dataset.key) setDropdown(dd, b.dataset.value);
    saveSettings();
    if (seg.dataset.key === "ui_language") applyLanguage(b.dataset.value);
    if (seg.dataset.key === "input_type") paintWelcome();
  };
}
// Tùy chọn giao diện dạng segment (sáng/tối, chất liệu): lưu qua glass.js, không vào config.toml.
const localSegs = [...document.querySelectorAll(".seg[data-local]")];
function paintLocalSeg(seg, value) {
  for (const b of seg.querySelectorAll("button")) b.classList.toggle("active", b.dataset.value === value);
  paintHint(seg, value);
}
for (const seg of localSegs) {
  seg.onclick = (e) => {
    const b = e.target.closest("button");
    if (!b) return;
    paintLocalSeg(seg, b.dataset.value);
    lookSet(seg.dataset.local, b.dataset.value);
  };
}
for (const k of BOOLS) $(k).onchange = saveSettings;
$("excluded_apps").onchange = saveSettings;
for (const id of WORD_LISTS) $(id).onchange = saveSettings;
$("save-macros").onclick = saveMacros;
$("add-last-app").onclick = addLastApp;
// A letter is added as soon as it is typed, so there is no add button or Enter handling.
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
// Turning it on restarts EVKey elevated (this window closes and reopens); turning it off applies at the next start.
$("admin").onchange = async () => {
  const on = $("admin").checked;
  const ok = await run(() => invoke("set_admin", { on }), t(on ? "toast.admin_on" : "toast.admin_off"));
  if (!ok) $("admin").checked = !on;
};
$("open-folder").onclick = () => run(() => invoke("open_config_dir"), t("toast.open_folder_ok"));
$("reset").onclick = resetConfig;
$("reset-broken").onclick = resetConfig;

// Ô phím tắt (`button.rec[data-key]`): bấm để ghi, rồi nhấn tổ hợp mới. Esc hủy; `.rec-clear` đặt về none.
// Cú pháp cấu hình: phím bổ trợ theo thứ tự ctrl, alt, shift, win, rồi một phím ("alt+z").
// Chỉ ô có data-modonly (phím bật/tắt) nhận tổ hợp chỉ gồm phím bổ trợ ("ctrl+shift").
const HOTKEY_NAMES = { ctrl: "Ctrl", alt: "Alt", shift: "Shift", win: "Win", space: "Space" };
const HOTKEY_BTNS = [...document.querySelectorAll("button.rec[data-key]")];
const MODIFIER_KEYS = ["Control", "Shift", "Alt", "Meta"];
let recordingHotkey = null;
let heldMods = []; // phím bổ trợ đã giữ trong lượt ghi hiện tại
let sawKey = false; // lượt ghi hiện tại đã có phím thường
function clearBtnOf(btn) {
  return document.querySelector(`.rec-clear[data-for="${btn.dataset.key}"]`);
}
function paintHotkey() {
  for (const btn of HOTKEY_BTNS) {
    const value = state?.settings[btn.dataset.key] ?? "none";
    btn.classList.toggle("recording", recordingHotkey === btn);
    btn.replaceChildren();
    if (recordingHotkey === btn) btn.textContent = t("keys.rec.prompt");
    else if (value === "none") btn.textContent = t("keys.rec.unset");
    else for (const part of value.split("+")) btn.append(Object.assign(document.createElement("kbd"), { textContent: HOTKEY_NAMES[part] ?? part.toUpperCase() }));
    const clear = clearBtnOf(btn);
    if (clear) clear.disabled = value === "none";
  }
}
// Thông báo lỗi nằm ngay dưới hàng phím tắt (`.rec-msg[data-for]`).
function recMsg(btn, text = "") {
  const el = document.querySelector(`.rec-msg[data-for="${btn.dataset.key}"]`);
  if (el) el.textContent = text;
}
function rowName(btn) {
  return btn.dataset.name ?? btn.closest(".row")?.querySelector("b")?.textContent ?? btn.dataset.key;
}
// Key name in the config grammar (letter, digit, f1-f12, space) for a keydown event, or null.
function hotkeyKey(e) {
  let m;
  if ((m = e.code.match(/^Key([A-Z])$/))) return m[1].toLowerCase();
  if ((m = e.code.match(/^Digit(\d)$/))) return m[1];
  if ((m = e.code.match(/^F(\d+)$/)) && m[1] >= 1 && m[1] <= 12) return "f" + Number(m[1]);
  return e.code === "Space" ? "space" : null;
}
function modsOf(e) {
  return [e.ctrlKey && "ctrl", e.altKey && "alt", e.shiftKey && "shift", e.metaKey && "win"].filter(Boolean);
}
async function setHotkey(btn, value) {
  if (value !== "none") {
    const other = HOTKEY_BTNS.find((b) => b !== btn && state.settings[b.dataset.key] === value);
    if (other) return recMsg(btn, t("keys.rec.used", { name: rowName(other) }));
  }
  recordingHotkey = null;
  recMsg(btn);
  state.settings[btn.dataset.key] = value;
  await saveSettings();
  paintHotkey();
}
for (const btn of HOTKEY_BTNS) {
  btn.onclick = () => {
    recordingHotkey = btn;
    heldMods = [];
    sawKey = false;
    recMsg(btn);
    paintHotkey();
  };
  btn.onblur = () => {
    recordingHotkey = null;
    recMsg(btn);
    paintHotkey();
  };
  btn.onkeydown = (e) => {
    if (recordingHotkey !== btn) return;
    e.preventDefault();
    if (e.key === "Escape") {
      recordingHotkey = null;
      recMsg(btn);
      paintHotkey();
      return;
    }
    if (MODIFIER_KEYS.includes(e.key)) {
      const mods = modsOf(e);
      if (mods.length === 1) {
        heldMods = mods;
        sawKey = false;
      } else if (mods.length > heldMods.length) heldMods = mods;
      return;
    }
    sawKey = true;
    const key = hotkeyKey(e);
    if (!key) return recMsg(btn, t("translate.hotkey.bad_key"));
    const mods = modsOf(e);
    if (!mods.length) return recMsg(btn, t("keys.rec.need_modifier"));
    setHotkey(btn, [...mods, key].join("+"));
  };
  btn.onkeyup = (e) => {
    if (recordingHotkey !== btn || sawKey || !MODIFIER_KEYS.includes(e.key) || !btn.hasAttribute("data-modonly")) return;
    if (heldMods.length < 2) return recMsg(btn, t("keys.rec.need_two_mods"));
    setHotkey(btn, heldMods.join("+"));
  };
}
for (const clear of document.querySelectorAll(".rec-clear[data-for]")) {
  clear.onclick = () => {
    const btn = HOTKEY_BTNS.find((b) => b.dataset.key === clear.dataset.for);
    if (btn) setHotkey(btn, "none");
  };
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

// Dropdowns bound to a config field (options with several long names, or that are rarely changed).
function setDropdown(dd, value) {
  dd.value = value;
  paintHint(dd, value);
}
for (const dd of DDS) {
  initDropdown(dd);
  dd.onchange = () => {
    paintHint(dd, dd.value);
    // The welcome screen keeps a segmented copy of the input method; keep it in step.
    for (const seg of SEGS) if (seg.dataset.key === dd.dataset.key) setSeg(seg, dd.value);
    saveSettings();
    if (dd.dataset.key === "input_type") paintWelcome();
    updateVisibility();
  };
}

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

// Mục "Nâng cao": mở và đóng bằng hoạt ảnh chiều cao + độ mờ; trạng thái mở được nhớ theo từng trang.
for (const d of document.querySelectorAll("details.adv")) {
  const body = d.querySelector(".adv-body");
  const key = "evkey-adv-" + d.dataset.adv;
  d.open = localStorage.getItem(key) === "1";
  d.querySelector(":scope > summary").onclick = (e) => {
    e.preventDefault();
    // Đang đóng dở thì `_closing` giữ mục mở tới hết hoạt ảnh; bấm lại sẽ mở ngay từ vị trí hiện tại.
    const opening = !d.open || d._closing;
    const from = d.open ? { h: body.offsetHeight, o: Number(getComputedStyle(body).opacity) } : { h: 0, o: 0 };
    localStorage.setItem(key, opening ? "1" : "0");
    d._closing = !opening;
    if (opening) d.open = true;
    if (reduceMotion.matches) {
      stopAnim(body);
      d.open = opening;
      d._closing = false;
      return;
    }
    animateBox(
      body,
      opening,
      opening ? 320 : 240,
      () => {
        if (!opening) d.open = false;
        d._closing = false;
      },
      from,
    );
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

// data-when: "key", "key=value", "key!=value" theo khóa cấu hình; "local:<tên>=value" đọc tùy chọn giao diện (glass.js).
function currentValue(key) {
  if (key.startsWith("local:")) return lookGet(key.slice(6));
  const control = document.querySelector(`.dd[data-key="${key}"]`);
  if (control) return control.value;
  const seg = document.querySelector(`.seg[data-key="${key}"]`);
  if (seg) return seg.querySelector(".active")?.dataset.value;
  return $(key)?.checked;
}

// `.content` mang class `animating` khi đang có hoạt ảnh; theme.css bỏ backdrop-filter lúc đó vì relayout kính
// mỗi khung hình gây giật trên GPU tích hợp.
const contentEl = document.querySelector(".content");
const animRunning = new Set();
function syncAnimating() {
  contentEl.classList.toggle("animating", animRunning.size > 0);
}
function stopAnim(el) {
  if (el._anim) animRunning.delete(el._anim);
  el._anim?.cancel();
  el._anim = null;
  el.style.height = "";
  el.style.overflow = "";
  syncAnimating();
}
// Animates height + opacity to shown/hidden, starting from `from` (default: the current frame) so an interrupted
// animation reverses smoothly. onDone runs only when the animation finishes, never when it is interrupted.
function animateBox(el, show, duration, onDone, from) {
  const fromH = from?.h ?? el.offsetHeight;
  const fromO = from?.o ?? Number(getComputedStyle(el).opacity);
  stopAnim(el);
  el.style.overflow = "hidden";
  el.style.height = "auto";
  const toH = show ? el.offsetHeight : 0;
  const anim = el.animate([{ height: fromH + "px", opacity: fromO }, { height: toH + "px", opacity: show ? 1 : 0 }], {
    duration,
    easing: "cubic-bezier(.32,.72,0,1)",
  });
  el._anim = anim;
  animRunning.add(anim);
  syncAnimating();
  anim.onfinish = () => {
    el._anim = null;
    animRunning.delete(anim);
    el.style.height = "";
    el.style.overflow = "";
    syncAnimating();
    onDone?.();
  };
}
// Hiện hoặc ẩn một mục data-when; lần đặt đầu tiên không có hoạt ảnh (xử lý trong updateVisibility).
function setWhen(el, show) {
  if (reduceMotion.matches) {
    stopAnim(el);
    el.hidden = !show;
    return;
  }
  const wasHidden = el.hidden;
  if (show) el.hidden = false;
  animateBox(el, show, show ? 320 : 240, () => (el.hidden = !show), show && wasHidden ? { h: 0, o: 0 } : undefined);
}

function updateVisibility() {
  for (const el of document.querySelectorAll("[data-platform], [data-when]")) {
    let hide = false;
    if (el.dataset.when) {
      const [, key, op, want] = el.dataset.when.match(/^((?:local:)?\w+)(?:(!?=)(.+))?$/);
      const value = currentValue(key);
      hide ||= op === "=" ? value !== want : op === "!=" ? value === want : !value;
    }
    const show = !hide;
    if (el.dataset.vis === undefined) el.hidden = hide;
    else if (el.dataset.vis !== String(show)) setWhen(el, show);
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

// Hero (tab Gõ tiếng Việt): chế độ hiện tại, phím tắt bật/tắt, công tắc V/E. popup_state trả chế độ thực tế;
// popup_toggle_mode đảo chế độ (cùng lệnh với popup khay hệ thống).
function hotkeyText(value) {
  return value === "none" ? t("keys.rec.unset") : value.split("+").map((p) => HOTKEY_NAMES[p] ?? p.toUpperCase()).join(" + ");
}
async function paintHero() {
  const s = await invoke("popup_state").catch(() => null);
  const vi = s?.mode === "vietnamese";
  const input = document.querySelector('.dd[data-key="input_type"] .dd-btn')?.firstElementChild?.textContent ?? "";
  $("hero-mode").textContent = `${t(vi ? "hero.vi" : "hero.en")} · ${input}`;
  $("hero-toggle").checked = vi;
  $("hero-hotkey").textContent = hotkeyText(state?.settings.toggle_hotkey ?? "none");
}
$("hero-toggle").onchange = async () => {
  try {
    await invoke("popup_toggle_mode");
  } catch (e) {
    say(String(e), "err");
  }
  paintHero();
};
// Nền mesh của hero tạm dừng khi cửa sổ mất tiêu điểm (theme.css đọc <html data-focus>).
const setFocusAttr = (on) => document.documentElement.toggleAttribute("data-focus", on && !document.hidden);
addEventListener("focus", () => {
  setFocusAttr(true);
  paintHero();
});
addEventListener("blur", () => setFocusAttr(false));
document.addEventListener("visibilitychange", () => setFocusAttr(document.hasFocus()));
setFocusAttr(document.hasFocus());

paintLook();
placePill(currentTab, true);
new ResizeObserver(() => placePill(currentTab, true)).observe(navPill.parentElement);
