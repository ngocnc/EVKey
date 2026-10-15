// Tùy chọn hiển thị dùng chung cho mọi cửa sổ EVKey (cùng origin nên chung localStorage).
// Chỉ lưu trên máy này, không vào config.toml. Gắn lên <html>: data-accent, data-material, --tr (0-1).
const LOOK_DEFAULTS = { accent: "ocean", material: "glass", transparency: "60", theme: "system" };
const LOOK_ACCENTS = ["ocean", "orchid", "sunset", "mint", "graphite"];
const lookGet = (name) => localStorage.getItem("evkey-" + name) ?? LOOK_DEFAULTS[name];

function lookSet(name, value) {
  localStorage.setItem("evkey-" + name, String(value));
  lookApply();
  if (name === "theme") lookTheme();
}

// Đưa tùy chọn về mặc định; các khóa cũ evkey-glass-* (đã bỏ) cũng bị xóa.
function lookReset() {
  for (const name of Object.keys(LOOK_DEFAULTS)) localStorage.removeItem("evkey-" + name);
  for (const k of Object.keys(localStorage)) if (k.startsWith("evkey-glass-")) localStorage.removeItem(k);
  lookApply();
  lookTheme();
}

function lookApply() {
  const root = document.documentElement;
  const accent = lookGet("accent");
  root.dataset.accent = LOOK_ACCENTS.includes(accent) ? accent : "ocean";
  root.dataset.material = lookGet("material") === "plain" ? "plain" : "glass";
  root.dataset.themePref = lookGet("theme");
  const pct = Number(lookGet("transparency"));
  root.style.setProperty("--tr", (Number.isFinite(pct) ? Math.min(100, Math.max(0, pct)) : 60) / 100);
}

// Sáng/tối theo lựa chọn; "system" theo Windows. Trang không chạy trong Tauri (xem trước trên trình duyệt) thì bỏ qua.
function lookTheme() {
  const theme = lookGet("theme");
  const win = window.__TAURI__?.window.getCurrentWindow();
  win?.setTheme(theme === "system" ? null : theme).catch(console.error);
}

lookApply();
lookTheme();
addEventListener("storage", (e) => {
  if (!e.key?.startsWith("evkey-")) return;
  lookApply();
  if (e.key === "evkey-theme") lookTheme();
});
