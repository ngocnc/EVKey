const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const $ = (id) => document.getElementById(id);
const BOOLS = [
  "free_mark", "check_spelling", "restore_if_wrong_spelling", "quick_telex",
  "quick_start_consonant", "quick_end_consonant", "upper_case_first_char",
  "allow_consonant_zfwj", "auto_caps_macro",
];
const SELECTS = ["input_type", "tone_placement", "code_table", "temp_off_key"];
let state = null;

function say(text, cls = "") {
  $("msg").textContent = text;
  $("msg").className = "msg " + cls;
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

function fill() {
  const s = state.settings;
  for (const k of BOOLS) $(k).checked = s[k];
  for (const k of SELECTS) $(k).value = s[k];
  $("toggle_hotkey").value = s.toggle_hotkey;
  $("excluded_apps").value = s.excluded_apps.join("\n");
  $("macro-rows").replaceChildren();
  for (const [k, v] of Object.entries(state.macros)) addMacroRow(k, v);
  if (!Object.keys(state.macros).length) addMacroRow();
  $("autostart").checked = state.autostart;
  $("config-dir").textContent = state.config_dir;
  $("version").textContent = "v" + state.version;
  $("config-error").hidden = !state.config_error;
  $("config-error-text").textContent = state.config_error ?? "";
  $("toggle-keyboard").textContent = state.enabled ? "Gỡ khỏi danh sách" : "Thêm EVKey vào danh sách bàn phím";
}

function readForm() {
  const s = { ...state.settings };
  for (const k of BOOLS) s[k] = $(k).checked;
  for (const k of SELECTS) s[k] = $(k).value;
  s.toggle_hotkey = $("toggle_hotkey").value.trim();
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

async function saveSettings() {
  try {
    await invoke("save_settings", { settings: readForm() });
    $("hotkey-error").hidden = true;
    say("Đã lưu cài đặt.", "ok");
    await load();
  } catch (e) {
    const text = String(e);
    // Hotkey errors come from the backend with this prefix; show them under the field.
    const hotkey = text.startsWith("Phím tắt");
    $("hotkey-error").hidden = !hotkey;
    $("hotkey-error").textContent = hotkey ? text : "";
    say(hotkey ? "Chưa lưu cài đặt. Sửa phím tắt rồi lưu lại." : text, "err");
  }
}

async function saveMacros() {
  const rows = [...document.querySelectorAll(".macro-row")]
    .map((r) => ({ key: r.querySelector(".mk").value, value: r.querySelector(".mv").value }))
    .filter((r) => r.key.trim() || r.value.trim());
  if (await run(() => invoke("save_macros", { rows }), "Đã lưu macro.")) await load();
}

async function resetConfig() {
  if (!confirm("Đặt lại toàn bộ cài đặt và macro về mặc định? Nội dung hiện tại trong config.toml và macros.toml sẽ bị ghi đè.")) return;
  if (await run(() => invoke("reset_config"), "Đã đặt lại về mặc định.")) await load();
}

async function addLastApp() {
  try {
    const app = await invoke("last_app");
    if (!app) return say("Chưa ghi nhận ứng dụng nào. Hãy chuyển sang ứng dụng cần loại trừ rồi thử lại.");
    const list = lines($("excluded_apps").value);
    if (!list.some((x) => x.toLowerCase() === app.toLowerCase())) list.push(app);
    $("excluded_apps").value = list.join("\n");
    say(`Đã thêm ${app}. Bấm "Lưu cài đặt" để áp dụng.`, "ok");
  } catch (e) {
    say(String(e), "err");
  }
}

$("save-settings").onclick = saveSettings;
$("add-macro").onclick = () => addMacroRow();
$("save-macros").onclick = saveMacros;
$("add-last-app").onclick = addLastApp;
$("autostart").onchange = () =>
  run(() => invoke("set_autostart", { on: $("autostart").checked }), "Đã cập nhật tự chạy cùng Windows.");
$("open-folder").onclick = () => run(() => invoke("open_config_dir"), "Đã mở thư mục cấu hình.");
$("reset").onclick = resetConfig;
$("reset-broken").onclick = resetConfig;
$("open-github").onclick = () => run(() => invoke("open_github"), "Đã mở trang GitHub.");
$("toggle-keyboard").onclick = async () => {
  if (await run(() => invoke("set_enabled", { enabled: !state.enabled }), "Đã cập nhật danh sách bàn phím.")) await load();
};

listen("settings-show", load);
load();
