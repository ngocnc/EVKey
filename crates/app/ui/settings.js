const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const $ = (id) => document.getElementById(id);
const BOOLS = [
  "free_mark", "check_spelling", "restore_if_wrong_spelling", "quick_telex",
  "quick_start_consonant", "quick_end_consonant", "upper_case_first_char",
  "allow_consonant_zfwj", "auto_caps_macro",
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

function fill() {
  const s = state.settings;
  for (const k of BOOLS) $(k).checked = s[k];
  for (const seg of SEGS) setSeg(seg, s[seg.dataset.key]);
  $("excluded_apps").value = s.excluded_apps.join("\n");
  $("macro-rows").replaceChildren();
  for (const [k, v] of Object.entries(state.macros)) addMacroRow(k, v);
  if (!Object.keys(state.macros).length) addMacroRow();
  $("autostart").checked = state.autostart;
  $("config-dir").textContent = state.config_dir;
  $("version").textContent = "v" + state.version;
  $("config-error").hidden = !state.config_error;
  $("config-error-text").textContent = state.config_error ?? "";
  $("toggle-keyboard").textContent = state.enabled ? "Gỡ khỏi danh sách" : "Thêm vào danh sách";
  updateVisibility();
}

function readForm() {
  const s = { ...state.settings };
  for (const k of BOOLS) s[k] = $(k).checked;
  for (const seg of SEGS) s[seg.dataset.key] = seg.querySelector(".active")?.dataset.value ?? s[seg.dataset.key];
  s.excluded_apps = lines($("excluded_apps").value);
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
for (const [index, tab] of tabs.entries()) {
  tab.onclick = () => {
    if (index === currentTab) return;
    for (const t of tabs) t.classList.toggle("active", t === tab);
    for (const p of document.querySelectorAll(".pane")) {
      const show = p.dataset.pane === tab.dataset.tab;
      p.hidden = !show;
      p.classList.toggle("from-left", index < currentTab);
    }
    currentTab = index;
  };
}

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
$("autostart").onchange = () =>
  run(() => invoke("set_autostart", { on: $("autostart").checked }), "Đã cập nhật tự chạy cùng Windows");
$("open-folder").onclick = () => run(() => invoke("open_config_dir"), "Đã mở thư mục cấu hình");
$("reset").onclick = resetConfig;
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
  for (const el of document.querySelectorAll("[data-when]")) {
    const [, key, op, want] = el.dataset.when.match(/^(\w+)(?:(!?=)(.+))?$/);
    const value = currentValue(key);
    el.hidden = op === "=" ? value !== want : op === "!=" ? value === want : !value;
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
backdrop.value = localStorage.getItem("evkey-backdrop") ?? "60";
applyBackdrop();
backdrop.oninput = () => {
  localStorage.setItem("evkey-backdrop", backdrop.value);
  applyBackdrop();
};
