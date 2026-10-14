const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const $ = (id) => document.getElementById(id);
const applyTheme = () => {
  const th = localStorage.getItem("evkey-theme") ?? "system";
  window.__TAURI__.window.getCurrentWindow().setTheme(th === "system" ? null : th).catch(console.error);
};
applyTheme();

let fill = null;

async function render() {
  const v = await invoke("translate_state");
  setUiLanguage(v.ui_language);
  const loading = v.status === "loading";
  const text = $("tr-text");
  text.textContent = loading ? t("translate.loading") : v.text;
  text.className = "tr-text " + (loading ? "loading" : v.status === "error" ? "error" : "");
  // The countdown bar starts when the result is shown and runs for the configured time.
  fill?.cancel();
  // Hidden while loading and when the popup never closes by itself; it keeps its space so the text does not jump.
  const timed = !loading && v.ms > 0;
  $("tr-bar").style.visibility = timed ? "visible" : "hidden";
  if (timed) fill = $("tr-fill").animate([{ transform: "scaleX(1)" }, { transform: "scaleX(0)" }], { duration: v.ms, easing: "linear", fill: "forwards" });
  requestAnimationFrame(() => invoke("translate_fit", { height: Math.ceil($("panel").getBoundingClientRect().height) }));
}

// The popup never takes focus, so there is no Esc: a click closes it.
document.addEventListener("click", () => invoke("translate_close"));
listen("translate-update", render);
render();
