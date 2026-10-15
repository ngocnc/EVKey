// Fills a `.sec-banner` element. Màu lấy từ token theme.css; chỉ độ trong suốt (banner_opacity) còn đi qua CSS.
// Blur do Acrylic phía Rust xử lý (banner.rs), không liên quan trang này.
const BANNER_HTML = `
  <div class="lock"><svg viewBox="0 0 24 24" fill="currentColor"><path d="M12 2.5 1.6 20.5h20.8L12 2.5Zm-1 6.5h2v6h-2V9Zm0 8h2v2h-2v-2Z" fill-rule="evenodd"/></svg></div>
  <div><b></b><span></span></div>`;

// `s` holds { app, opacity }; an empty `app` means the holder is unknown.
function renderBanner(el, s) {
  if (!el.firstElementChild) el.innerHTML = BANNER_HTML;
  el.style.setProperty("--bn-opacity", Math.min(100, Math.max(0, s.opacity)) / 100);
  const app = s.app ? s.app[0].toUpperCase() + s.app.slice(1) : t("banner.unknown_app");
  el.querySelector("b").textContent = t("banner.title");
  el.querySelector("span").textContent = t("banner.blocking", { app });
}
