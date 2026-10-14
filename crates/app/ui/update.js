const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const $ = (id) => document.getElementById(id);
let busy = false;

function showError(msg) {
  $("update-error").textContent = msg;
  $("update-error").hidden = !msg;
}

async function load() {
  const [settings, release] = await Promise.all([invoke("get_state").then((s) => s.settings), invoke("update_pending")]);
  setUiLanguage(settings.ui_language);
  applyI18n();
  if (!release) return;
  $("update-version").textContent = t("update.version", { version: release.version });
  $("update-notes").textContent = release.notes || t("update.no_notes");
  showError("");
}

async function act(cmd) {
  if (busy) return;
  busy = true;
  $("install").disabled = true;
  if (cmd === "update_install") $("install").textContent = t("update.downloading");
  try {
    await invoke(cmd);
  } catch (e) {
    showError(String(e));
  }
  $("install").textContent = t("update.install");
  $("install").disabled = false;
  busy = false;
}

$("install").onclick = () => act("update_install");
$("later").onclick = () => act("update_later");
$("skip").onclick = () => act("update_skip");
listen("update-show", load);
applyI18n();
load();
