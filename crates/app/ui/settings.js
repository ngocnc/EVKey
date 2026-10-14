const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const $ = (id) => document.getElementById(id);
const BOOLS = [
  "free_mark", "remember_app_mode", "undo_word", "check_spelling", "restore_if_wrong_spelling", "quick_telex",
  "quick_start_consonant", "quick_end_consonant", "upper_case_first_char",
  "allow_consonant_zfwj", "auto_caps_macro", "toggle_sound",
];
const SEGS = [...document.querySelectorAll(".seg")];
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

function addMacroRow(key = "", value = "") {
  const row = document.createElement("div");
  row.className = "macro-row";
  const del = Object.assign(document.createElement("button"), { className: "ghost", textContent: "Xóa" });
  del.onclick = () => row.remove();
  row.append(
    Object.assign(document.createElement("input"), { className: "mk", placeholder: "Từ viết tắt", value: key }),
    Object.assign(document.createElement("input"), { className: "mv", placeholder: "Nội dung thay thế", value }),
    del,
  );
  $("macro-rows").append(row);
}

function renderCustomConsonants(list) {
  $("custom-consonants").replaceChildren(
    ...list.map((k) => {
      const chip = Object.assign(document.createElement("span"), { className: "chip" });
      const del = Object.assign(document.createElement("button"), { className: "chip-x", textContent: "×", title: "Xóa" });
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
    say("Đã lưu", "ok");
  } catch (e) {
    say(String(e), "err");
  }
}

async function addCustomConsonant() {
  const input = $("custom-consonant-input");
  const k = input.value.trim().toLowerCase();
  if (!/^[a-z]$/.test(k)) return say("Phụ âm phải là đúng 1 chữ cái a–z.", "err");
  if (state.settings.custom_consonants.includes(k)) return say(`Phụ âm "${k}" đã có.`, "err");
  input.value = "";
  await setCustomConsonants([...state.settings.custom_consonants, k]);
}

function setSeg(seg, value) {
  // A hotkey written by hand in config.toml that is not one of the presets gets its own button, so saving never rewrites it.
  if (seg.dataset.key === "toggle_hotkey") {
    seg.querySelector(".extra")?.remove();
    if (![...seg.children].some((b) => b.dataset.value === value)) {
      seg.append(Object.assign(document.createElement("button"), { className: "extra", textContent: value === "none" ? "Tắt" : value }));
      seg.lastChild.dataset.value = value;
    }
  }
  for (const b of seg.children) b.classList.toggle("active", b.dataset.value === value);
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
  for (const k of BOOLS) $(k).checked = s[k];
  for (const seg of SEGS) setSeg(seg, s[seg.dataset.key]);
  $("excluded_apps").value = s.excluded_apps.join("\n");
  $("macro-rows").replaceChildren();
  for (const [k, v] of Object.entries(state.macros)) addMacroRow(k, v);
  if (!Object.keys(state.macros).length) addMacroRow();
  renderCustomConsonants(s.custom_consonants);
  $("autostart").checked = state.autostart;
  $("config-dir").textContent = state.config_dir;
  $("version").textContent = "v" + state.version;
  $("about-version").textContent = "v" + state.version;
  $("config-error").hidden = !state.config_error;
  $("config-error-text").textContent = state.config_error ?? "";
  $("toggle-keyboard").textContent = state.enabled ? "Gỡ khỏi danh sách" : "Thêm vào danh sách";
  $("excluded_apps").placeholder = "game.exe";
  fillBanner(s);
  updateVisibility();
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
    say("Đã lưu", "ok");
  } catch (e) {
    say(String(e), "err");
  }
}

async function saveMacros() {
  const rows = [...document.querySelectorAll(".macro-row")]
    .map((r) => ({ key: r.querySelector(".mk").value, value: r.querySelector(".mv").value }))
    .filter((r) => r.key.trim() || r.value.trim());
  if (await run(() => invoke("save_macros", { rows }), "Đã lưu gõ tắt")) await load();
}

async function resetConfig() {
  if (!confirm("Đặt lại toàn bộ cài đặt và gõ tắt về mặc định? Nội dung hiện tại trong config.toml và macros.toml sẽ bị ghi đè.")) return;
  if (await run(() => invoke("reset_config"), "Đã đặt lại về mặc định")) await load();
}

async function addLastApp() {
  try {
    const app = await invoke("last_app");
    if (!app) return say("Chưa ghi nhận ứng dụng nào. Hãy chuyển sang ứng dụng cần loại trừ rồi thử lại.");
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

// Glass transparency is a look-only preference, kept in the window's local storage rather than config.toml.
const range = $("glass-range");
function applyGlass(percent) {
  document.documentElement.style.setProperty("--t", percent / 100);
  $("glass-value").textContent = percent + "%";
}
range.value = localStorage.getItem("evkey-glass") ?? "100";
applyGlass(Number(range.value));
range.oninput = () => {
  applyGlass(Number(range.value));
  localStorage.setItem("evkey-glass", range.value);
};
for (const seg of SEGS) {
  seg.onclick = (e) => {
    const b = e.target.closest("button");
    if (!b) return;
    for (const x of seg.children) x.classList.toggle("active", x === b);
    saveSettings();
  };
}
for (const k of BOOLS) $(k).onchange = saveSettings;
$("excluded_apps").onchange = saveSettings;
$("add-macro").onclick = () => addMacroRow();
$("save-macros").onclick = saveMacros;
$("add-last-app").onclick = addLastApp;
$("add-custom-consonant").onclick = addCustomConsonant;
$("custom-consonant-input").onkeydown = (e) => e.key === "Enter" && addCustomConsonant();
const appWindow = window.__TAURI__.window.getCurrentWindow();
// Settings window starts pinned; keep the real window level in sync with the pin button.
appWindow.setAlwaysOnTop(true).catch((e) => say(String(e), "err"));
$("pin").onclick = async () => {
  const pin = $("pin");
  const on = !pin.classList.contains("active");
  try {
    await appWindow.setAlwaysOnTop(on);
    pin.classList.toggle("active", on);
    pin.setAttribute("aria-pressed", String(on));
    pin.title = on ? "Bỏ ghim" : "Ghim lên trên cùng";
  } catch (e) {
    say(String(e), "err");
  }
};
$("autostart").onchange = () =>
  run(() => invoke("set_autostart", { on: $("autostart").checked }), "Đã cập nhật tự chạy cùng Windows");
$("open-folder").onclick = () => run(() => invoke("open_config_dir"), "Đã mở thư mục cấu hình");
$("reset").onclick = resetConfig;
for (const id of BANNER_FIELDS) {
  $(id).oninput = () => renderBannerPreview();
  $(id).onchange = saveSettings;
}
$("bn-reset").onclick = async () => {
  const d = await invoke("banner_defaults");
  $("bn-opacity").value = d.opacity;
  $("bn-blur").value = d.blur;
  $("bn-bg").value = d.bg;
  $("bn-border").value = d.border;
  $("bn-text").value = d.text;
  renderBannerPreview();
  saveSettings();
};
$("reset-broken").onclick = resetConfig;
$("open-github").onclick = () => run(() => invoke("open_github"), "Đã mở trang GitHub");
$("toggle-keyboard").onclick = async () => {
  if (await run(() => invoke("set_enabled", { enabled: !state.enabled }), "Đã cập nhật danh sách bàn phím")) await load();
};

listen("settings-show", load);
load();

// Items marked data-when ("key", "key=value" or "key!=value") only show while that option is chosen.
function currentValue(key) {
  const seg = document.querySelector(`.seg[data-key="${key}"]`);
  if (seg) return seg.querySelector(".active")?.dataset.value;
  return $(key)?.checked;
}

function updateVisibility() {
  for (const el of document.querySelectorAll("[data-platform], [data-when]")) {
    let hide = false;
    if (el.dataset.when) {
      const [, key, op, want] = el.dataset.when.match(/^(\w+)(?:(!?=)(.+))?$/);
      const value = currentValue(key);
      hide ||= op === "=" ? value !== want : op === "!=" ? value === want : !value;
    }
    el.hidden = hide;
  }
  for (const group of document.querySelectorAll(".group")) {
    const visible = [...group.querySelectorAll(".item")].filter((i) => !i.hidden);
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
for (const seg of SEGS) {
  seg.prepend(Object.assign(document.createElement("i"), { className: "thumb" }));
  thumbObserver.observe(seg);
  seg.addEventListener("click", () => placeThumb(seg));
}

// Backdrop transparency: a tint layer over the (always clearest) acrylic; 100 = fully clear.
const backdrop = $("backdrop-range");
function applyBackdrop() {
  $("backdrop-value").textContent = backdrop.value + "%";
  document.documentElement.style.setProperty("--bt", backdrop.value / 100);
}
backdrop.value = localStorage.getItem("evkey-backdrop") ?? "85";
applyBackdrop();
backdrop.oninput = () => {
  localStorage.setItem("evkey-backdrop", backdrop.value);
  applyBackdrop();
};
