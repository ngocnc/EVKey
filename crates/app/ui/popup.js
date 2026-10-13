const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const $ = (id) => document.getElementById(id);
let state = null;

function showError(text) {
  $("error").hidden = !text;
  $("error").textContent = text || "";
}

function render() {
  const on = state.enabled;
  $("badge").textContent = on ? "V" : "–";
  $("badge").classList.toggle("on", on);
  $("status-title").textContent = on ? "EVKey có trong danh sách bàn phím" : "EVKey chưa được thêm";
  $("status-hint").textContent = on ? "Chuyển Việt/Anh bằng Win+Space" : "Bật công tắc để thêm vào danh sách bàn phím";
  $("toggle").classList.toggle("on", on);
  $("toggle").setAttribute("aria-checked", String(on));
  $("input-type").value = state.settings.input_type;
  $("version").textContent = "v" + state.version;
}

async function load() {
  try {
    state = await invoke("get_state");
    render();
    showError(state.config_error ?? "");
  } catch (e) {
    showError(String(e));
  }
}

async function run(action) {
  try {
    await action();
    showError("");
  } catch (e) {
    showError(String(e));
  }
  await load();
}

$("toggle").onclick = () => run(() => invoke("set_enabled", { enabled: !state.enabled }));
$("input-type").onchange = () =>
  run(() => invoke("save_settings", { settings: { ...state.settings, input_type: $("input-type").value } }));
$("open-settings").onclick = () => invoke("show_settings");
$("quit").onclick = () => invoke("quit");

listen("popup-show", load);
load();
