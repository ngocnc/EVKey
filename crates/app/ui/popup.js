const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const $ = (id) => document.getElementById(id);
let state = null;

function showError(text) {
  $("error").hidden = !text;
  $("error").textContent = text || "";
}

function render() {
  const vi = state.mode === "vietnamese";
  const hotkey = state.settings.toggle_hotkey;
  $("badge").textContent = vi ? "V" : "E";
  $("badge").classList.toggle("on", vi);
  $("status-title").textContent = vi ? "Tiếng Việt" : "English";
  $("status-hint").textContent = hotkey === "none" ? "Không có phím tắt chuyển Việt/Anh" : `Phím tắt: ${hotkey}`;
  $("toggle").classList.toggle("on", vi);
  $("toggle").setAttribute("aria-checked", String(vi));
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

$("toggle").onclick = () => run(() => invoke("toggle_mode"));
$("input-type").onchange = () =>
  run(() => invoke("save_settings", { settings: { ...state.settings, input_type: $("input-type").value } }));
$("open-settings").onclick = () => invoke("show_settings");
$("quit").onclick = () => invoke("quit");

listen("popup-show", load);
load();
