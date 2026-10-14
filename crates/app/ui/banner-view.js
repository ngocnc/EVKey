// Fills a `.sec-banner` element from the banner_* settings. Used by the overlay window and the settings preview.
const BANNER_HTML = `
  <svg viewBox="0 0 24 24" fill="currentColor"><path d="M12 2.5 1.6 20.5h20.8L12 2.5Zm-1 6.5h2v6h-2V9Zm0 8h2v2h-2v-2Z" fill-rule="evenodd"/></svg>
  <div><b>Ô mật khẩu đang được nhập</b><span></span></div>`;

function hexToRgba(hex, alpha) {
  const n = parseInt(hex.slice(1), 16);
  return `rgba(${(n >> 16) & 255}, ${(n >> 8) & 255}, ${n & 255}, ${alpha})`;
}

// `s` holds { app, opacity, blur, bg, border, text }; an empty `app` means the holder is unknown.
function renderBanner(el, s) {
  if (!el.firstElementChild) el.innerHTML = BANNER_HTML;
  el.style.setProperty("--bn-bg", hexToRgba(s.bg, s.opacity / 100));
  el.style.setProperty("--bn-border", s.border);
  el.style.setProperty("--bn-text", s.text);
  el.querySelector("span").textContent = `${s.app || "Một ứng dụng"} đang chặn EVKey xử lý Tiếng Việt`;
}
