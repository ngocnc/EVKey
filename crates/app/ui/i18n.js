// UI strings for the settings, popup, banner, update and translate windows.
// To add a string: add a key with its `vi` and `en` text below, then use it in HTML as
// data-i18n (textContent), data-i18n-html (innerHTML), data-i18n-title, data-i18n-placeholder
// or data-i18n-aria-label, or in JS as t("key", { param }). Brand names and examples stay untranslated.
// Style: short sentences, say what the option does, same word for the same thing in both languages
// (Chế độ = mode, Gõ tắt = macro, Ứng dụng loại trừ = excluded app).
const STRINGS = {
  "settings.title": { vi: "EVKey - Cài đặt", en: "EVKey - Settings" },
  "common.delete": { vi: "Xóa", en: "Delete" },
  "common.off": { vi: "Tắt", en: "Off" },
  "common.none": { vi: "Không dùng", en: "None" },
  "reset.defaults": { vi: "Đặt lại về mặc định", en: "Reset to defaults" },
  "reset.confirm": {
    vi: "Đặt lại toàn bộ cài đặt và gõ tắt về mặc định? Nội dung hiện có trong config.toml và macros.toml sẽ bị ghi đè.",
    en: "Reset all settings and macros to their defaults? The current config.toml and macros.toml will be overwritten.",
  },
  "toast.reset_ok": { vi: "Đã đặt lại về mặc định", en: "Reset to defaults" },
  "toast.macro_deleted": { vi: "Đã xóa gõ tắt", en: "Macro deleted" },
  "toast.macros_saved": { vi: "Đã lưu gõ tắt", en: "Macros saved" },
  "toast.autostart_ok": { vi: "Đã cập nhật tùy chọn tự chạy cùng Windows", en: "Start with Windows updated" },
  "toast.open_folder_ok": { vi: "Đã mở thư mục cấu hình", en: "Config folder opened" },
  "toast.open_github_ok": { vi: "Đã mở trang GitHub", en: "GitHub page opened" },
  "apps.none_recorded": {
    vi: "Chưa ghi nhận ứng dụng nào. Hãy chuyển sang ứng dụng muốn loại trừ rồi thử lại.",
    en: "No app recorded yet. Switch to the app you want to exclude, then try again.",
  },
  "custom.invalid": {
    vi: "Chỉ nhập một chữ cái a–z, không dấu. Nếu chữ bị đổi thành có dấu, hãy chuyển EVKey sang gõ tiếng Anh rồi thử lại.",
    en: "Enter a single letter a–z with no accent. If the letter turns into an accented one, switch EVKey to English and try again.",
  },
  "custom.duplicate": { vi: 'Phụ âm "{key}" đã có.', en: 'The consonant "{key}" is already in the list.' },
  "pin.unpin": { vi: "Bỏ ghim", en: "Unpin" },
  "pin.pin": { vi: "Ghim lên trên cùng", en: "Keep on top" },

  // Navigation tabs and pane headings
  "nav.typing": { vi: "Gõ tiếng Việt", en: "Vietnamese typing" },
  "nav.smart": { vi: "Chính tả & gõ nhanh", en: "Smart typing" },
  "nav.macro": { vi: "Gõ tắt", en: "Macros" },
  "nav.keys": { vi: "Phím tắt", en: "Hotkeys" },
  "nav.apps": { vi: "Ứng dụng loại trừ", en: "Excluded apps" },
  "nav.appearance": { vi: "Giao diện", en: "Appearance" },
  "nav.system": { vi: "Hệ thống", en: "System" },
  "nav.translate": { vi: "Dịch thuật", en: "Translate" },
  "nav.about": { vi: "Thông tin", en: "About" },

  // Translate popup and tab
  "translate.loading": { vi: "Đang dịch…", en: "Translating…" },
  "translate.hotkey.title": { vi: "Phím tắt dịch", en: "Translate hotkey" },
  "translate.hotkey.desc": {
    vi: "Bôi đen văn bản ở bất kỳ ứng dụng nào rồi bấm phím này. Tiếng Việt được dịch sang tiếng Anh, ngôn ngữ khác được dịch sang tiếng Việt.",
    en: "Select text in any app, then press this key. Vietnamese is translated to English; any other language is translated to Vietnamese.",
  },
  "translate.hotkey.recording": { vi: "Bấm phím tắt mới… (Esc: hủy)", en: "Press the new hotkey… (Esc to cancel)" },
  "translate.hotkey.unset": { vi: "Chưa đặt – bấm để đặt", en: "Not set – click to set" },
  "translate.hotkey.need_modifier": { vi: "Hãy giữ Ctrl, Alt hoặc Shift cùng với một phím.", en: "Hold Ctrl, Alt or Shift together with a key." },
  "translate.hotkey.bad_key": { vi: "Chỉ nhận chữ cái, chữ số, Space và F1–F12.", en: "Only letters, digits, Space and F1–F12 can be used." },
  "translate.action.title": { vi: "Sau khi dịch", en: "After translating" },
  "translate.action.desc": {
    vi: "Bản dịch luôn hiện trong một cửa sổ nhỏ. EVKey có thể tự sao chép nó, hoặc dán đè lên đoạn văn bản bạn đã bôi đen.",
    en: "The translation always shows in a small popup. EVKey can also copy it, or paste it over the text you selected.",
  },
  "translate.action.show": { vi: "Chỉ hiện", en: "Show only" },
  "translate.action.copy": { vi: "Sao chép", en: "Copy" },
  "translate.action.replace": {
    vi: "Dán đè lên văn bản",
    en: "Paste over selection",
  },
  "translate.secs.title": { vi: "Thời gian hiển thị", en: "Display time" },
  "translate.secs.desc": {
    vi: "Thời gian cửa sổ dịch hiện trước khi tự đóng (thanh phía trên chạy hết). Bấm vào cửa sổ để đóng ngay. Chọn Không tự đóng để giữ đến khi bạn bấm.",
    en: "How long the popup stays before closing; the bar at the top counts down. Click it to close it sooner. Choose Never close to keep it until you click.",
  },
  "translate.unit.s": { vi: "giây", en: "s" },
  "translate.unit.never": { vi: "Không tự đóng", en: "Never close" },
  "translate.provider.title": { vi: "Dịch vụ dịch", en: "Translation service" },
  "translate.provider.desc": {
    vi: "Văn bản bạn chọn được gửi tới Google qua internet mỗi lần dịch.",
    en: "The text you select is sent to Google over the internet each time you translate.",
  },

  // Typing pane
  "typing.input_type.title": { vi: "Kiểu gõ", en: "Input method" },
  "typing.input_type.desc": {
    vi: "Chọn cách gõ dấu. Telex: as → á. VNI: a1 → á.",
    en: "How you type tone marks. Telex: as → á. VNI: a1 → á.",
  },
  "typing.tone.title": {
    vi: "Vị trí đặt dấu",
    en: "Tone mark position",
  },
  "typing.tone.desc": {
    vi: "Với oa, oe, uy: kiểu mới đặt dấu ở chữ thứ hai (hòa), kiểu cũ ở chữ thứ nhất (hoà).",
    en: "For oa, oe and uy: the new style puts the mark on the second letter (hòa), the old style on the first (hoà).",
  },
  "typing.tone.new": { vi: "Kiểu mới (hòa)", en: "New style (hòa)" },
  "typing.tone.old": { vi: "Kiểu cũ (hoà)", en: "Old style (hoà)" },
  "typing.table.title": {
    vi: "Bảng mã",
    en: "Text encoding",
  },
  "typing.table.desc": {
    vi: "Cách lưu chữ có dấu. Nên giữ Unicode, trừ khi một ứng dụng cũ cần bảng mã khác.",
    en: "How accented letters are stored. Keep Unicode unless an old app needs another one.",
  },
  "table.unicode_compound": { vi: "Unicode tổ hợp", en: "Unicode compound" },
  "typing.free_mark.title": {
    vi: "Đặt dấu tự do",
    en: "Free tone placement",
  },
  "typing.free_mark.desc": {
    vi: "Cho phép gõ phím dấu sau khi đã gõ hết các chữ cái của từ.",
    en: "Lets you type the tone key after the rest of the word is already typed.",
  },

  // Hotkeys pane
  "keys.toggle.title": {
    vi: "Chuyển Việt / Anh",
    en: "Switch Vietnamese / English",
  },
  "keys.toggle.desc": {
    vi: "Bấm tổ hợp phím này để bật hoặc tắt gõ tiếng Việt, dùng được ở mọi ứng dụng.",
    en: "Press this to switch typing between Vietnamese and English in any app.",
  },
  "keys.temp_off.title": {
    vi: "Phím tạm tắt trong từ",
    en: "Pause key",
  },
  "keys.temp_off.desc": {
    vi: "Bấm phím này giữa chừng một từ để phần còn lại của từ được gõ nguyên văn, không bỏ dấu.",
    en: "Press this key in the middle of a word and the rest of that word is typed exactly as you press it, with no accents.",
  },
  "keys.note": {
    vi: "Windows có thể có phím tắt riêng để đổi bàn phím (<kbd>Ctrl</kbd><kbd>Shift</kbd> hoặc <kbd>Alt</kbd><kbd>Shift</kbd>). Đó không phải phím của EVKey.",
    en: "Windows may have its own hotkey for switching keyboards (<kbd>Ctrl</kbd><kbd>Shift</kbd> or <kbd>Alt</kbd><kbd>Shift</kbd>). That one does not belong to EVKey.",
  },

  // Spelling and quick typing pane
  "smart.spell.title": { vi: "Kiểm tra chính tả", en: "Spell check" },
  "smart.spell.desc": {
    vi: "Chỉ thêm dấu khi chữ vừa gõ có thể là một từ tiếng Việt.",
    en: "Only adds accents when what you typed can be a Vietnamese word.",
  },
  "smart.restore.title": {
    vi: "Giữ nguyên từ không phải tiếng Việt",
    en: "Keep non-Vietnamese words as typed",
  },
  "smart.restore.desc": {
    vi: "Gõ <code>window</code> sẽ ra <code>window</code>, không bị đổi thành chữ lạ.",
    en: "Typing <code>window</code> gives <code>window</code>, not odd letters.",
  },
  "smart.quick.heading": { vi: "Gõ nhanh", en: "Quick typing" },
  "smart.quick_telex.title": { vi: "Telex nhanh", en: "Quick Telex" },
  "smart.quick_start.title": { vi: "Phụ âm đầu nhanh", en: "Quick initial consonants" },
  "smart.quick_end.title": { vi: "Phụ âm cuối nhanh", en: "Quick final consonants" },
  "smart.custom.title": {
    vi: "Phụ âm thêm (mặc định Z, F, W, J)",
    en: "Extra consonants (default Z, F, W, J)",
  },
  "smart.custom.desc": {
    vi: "Cho phép các chữ này đứng đầu từ, dùng cho từ mượn và tên riêng, ví dụ Zalo, Wifi.",
    en: "Lets these letters start a word, for loanwords and names such as Zalo or Wifi.",
  },
  "smart.custom.add_aria": { vi: "Thêm phụ âm", en: "Add consonant" },
  "smart.custom.caption": {
    vi: "Các phụ âm này được phép dùng khi gõ tiếng Việt.",
    en: "These consonants are allowed when typing Vietnamese.",
  },
  "smart.app_mode.title": {
    vi: "Nhớ Việt/Anh theo ứng dụng",
    en: "Remember Vietnamese/English per app",
  },
  "smart.app_mode.desc": {
    vi: "Mỗi ứng dụng nhớ riêng bạn đang gõ tiếng Việt hay tiếng Anh. Khi quay lại ứng dụng đó, EVKey tự chuyển về đúng chế độ.",
    en: "Each app remembers whether you were typing Vietnamese or English. When you come back to it, EVKey switches to that mode again.",
  },
  "smart.undo.title": {
    vi: "Hoàn tác bằng Esc",
    en: "Undo with Esc",
  },
  "smart.undo.desc": {
    vi: "Đang gõ dở một từ, bấm Esc để trả từ đó về đúng các phím bạn đã bấm (vieejt thay vì việt).",
    en: "While typing a word, press Esc to get back the exact keys you pressed (vieejt instead of việt).",
  },
  "smart.caps.heading": { vi: "Viết hoa", en: "Capitalization" },
  "smart.caps_first.title": {
    vi: "Tự viết hoa đầu câu",
    en: "Capitalize sentences",
  },
  "smart.caps_first.desc": {
    vi: "Tự viết hoa chữ cái đầu tiên của câu mới. Chọn các trường hợp bên dưới.",
    en: "Capitalizes the first letter of a new sentence. Choose the cases below.",
  },
  "smart.caps_punct.title": { vi: "Sau các dấu câu . ? !", en: "After . ? ! marks" },
  "smart.caps_punct.desc": { vi: "xong rồi. tôi đi → xong rồi. Tôi đi", en: "done. i go → done. I go" },
  "smart.caps_enter.title": { vi: "Sau khi bấm Enter", en: "After pressing Enter" },
  "smart.caps_enter.desc": { vi: "Chữ cái đầu tiên của dòng mới.", en: "The first letter of the new line." },
    "smart.caps_first_letter.title": { vi: "Chữ cái đầu tiên", en: "First letter" },
  "smart.caps_first_letter.desc": {
    vi: "Chữ cái đầu tiên bạn gõ sau khi chuyển ứng dụng hoặc bấm chuột vào ô nhập.",
    en: "The first letter you type after switching apps or clicking into a text box.",
  },
  "smart.macro_case.title": {
    vi: "Gõ tắt theo kiểu chữ",
    en: "Match macro letter case",
  },
  "smart.macro_case.desc": {
    vi: "Nội dung thay thế viết hoa theo từ viết tắt: gõ Btw ra By the way, gõ BTW ra BY THE WAY.",
    en: "The replacement follows how you capitalize the abbreviation: Btw gives By the way, BTW gives BY THE WAY.",
  },

  // Macros pane
  "macro.note": {
    vi: "Gõ từ viết tắt rồi bấm Space, EVKey sẽ thay bằng nội dung bên phải. Mỗi từ viết tắt chỉ dùng được một lần.",
    en: "Type an abbreviation and press Space, and EVKey replaces it with the text on the right. Each abbreviation can be used only once.",
  },
  "macro.row_incomplete": {
    vi: "Dòng {n}: nhập cả từ viết tắt lẫn nội dung thay thế, hoặc xóa dòng này.",
    en: "Row {n}: enter both the abbreviation and its replacement text, or delete the row.",
  },
  "macro.save": { vi: "Lưu gõ tắt", en: "Save macros" },
  "macro.key_placeholder": { vi: "Từ viết tắt", en: "Abbreviation" },
  "macro.value_placeholder": { vi: "Nội dung thay thế", en: "Replacement text" },

  // Excluded apps pane
  "apps.note": {
    vi: "EVKey sẽ không gõ tiếng Việt trong các ứng dụng này, ví dụ game. Mỗi dòng một tên tệp, như <code>game.exe</code>.",
    en: "EVKey will not type Vietnamese in these apps, for example games. Put one file name per line, like <code>game.exe</code>.",
  },
  "apps.add_last": { vi: "Thêm ứng dụng vừa dùng", en: "Add the app I used last" },

  // Appearance pane
  "look.theme.title": { vi: "Chủ đề", en: "Theme" },
  "look.theme.desc": { vi: "Sáng, tối hoặc theo Windows.", en: "Light, dark, or the same as Windows." },
  "look.theme.system": { vi: "Theo Windows", en: "Same as Windows" },
  "look.theme.light": { vi: "Sáng", en: "Light" },
  "look.theme.dark": { vi: "Tối", en: "Dark" },
  "look.glass.title": { vi: "Kiểu kính", en: "Glass style" },
  "look.glass.desc": {
    vi: "Thường: làm mờ nền phía sau. Trong suốt: nhìn xuyên qua rõ hơn.",
    en: "Regular blurs what is behind the window. Clear lets you see it more sharply.",
  },
  "look.glass.regular": { vi: "Thường", en: "Regular" },
  "look.glass.clear": { vi: "Trong suốt", en: "Clear" },
  "look.strength.title": { vi: "Độ đậm màu phủ", en: "Tint strength" },
  "look.strength.desc": {
    vi: "Độ đậm của lớp màu phủ lên nền cửa sổ. Đặt 0 để bỏ.",
    en: "How strong the color layer over the window is. Set 0 to remove it.",
  },
  "look.tint": { vi: "Màu phủ", en: "Tint" },
  "look.accent.title": { vi: "Màu chủ đạo", en: "Accent color" },
  "look.accent.desc": {
    vi: "Màu của logo, công tắc, thanh trượt và thanh cuộn.",
    en: "Color of the logo, switches, sliders and scrollbars.",
  },
  "look.default": { vi: "Mặc định", en: "Default" },
  "look.banner.heading": { vi: "Cảnh báo Secure Input", en: "Secure Input warning" },
  "look.bn_opacity.title": { vi: "Độ đậm nền", en: "Background opacity" },
  "look.bn_blur.title": { vi: "Độ mờ kính", en: "Glass blur" },
  "look.bn_blur.desc": { vi: "Đặt 0 để tắt hiệu ứng kính.", en: "Set 0 to turn off the glass effect." },
  "look.bn_bg": { vi: "Màu nền", en: "Background color" },
  "look.bn_border": { vi: "Màu viền", en: "Border color" },
  "look.bn_text": { vi: "Màu chữ", en: "Text color" },
  "look.bn_hint": {
    vi: "Hiện ở cạnh dưới màn hình khi EVKey tạm tắt vì bạn đang nhập vào ô mật khẩu.",
    en: "Appears at the bottom of the screen when EVKey pauses because you are typing in a password field.",
  },
  "look.reset": { vi: "Đặt lại giao diện", en: "Reset appearance" },

  // Secure Input banner
  "banner.title": { vi: "EVKey tạm tắt trong ô mật khẩu", en: "EVKey is paused in a password field" },
  "banner.blocking": { vi: "{app} đang chặn EVKey gõ tiếng Việt", en: "{app} is blocking EVKey from typing Vietnamese" },
  "banner.unknown_app": { vi: "Một ứng dụng", en: "An app" },

  // System pane
  "system.ui_language.title": {
    vi: "Ngôn ngữ",
    en: "Language",
  },
  "system.ui_language.desc": {
    vi: "Ngôn ngữ hiển thị của các cửa sổ và thông báo EVKey. Không ảnh hưởng đến việc gõ tiếng Việt.",
    en: "The language EVKey's windows and messages are shown in. It does not affect Vietnamese typing.",
  },
  "system.autostart.title": { vi: "Tự chạy cùng Windows", en: "Start with Windows" },
  "system.autostart.desc": { vi: "EVKey tự mở và nằm ở khay hệ thống khi bạn đăng nhập.", en: "EVKey starts in the system tray when you sign in." },
  "system.sound.title": { vi: "Phát tiếng khi đổi chế độ", en: "Sound when switching mode" },
  "system.sound.desc": {
    vi: "Phát một tiếng ngắn khi bạn bật hoặc tắt gõ tiếng Việt.",
    en: "Plays a short sound when you turn Vietnamese typing on or off.",
  },
  "system.files.heading": { vi: "Tệp cấu hình", en: "Config files" },
  "system.open_folder": { vi: "Mở thư mục", en: "Open folder" },

  // About pane
  "about.intro": {
    vi: "Bộ gõ tiếng Việt cho Windows, dùng lõi xử lý chuyển từ OpenKey (GPL-3.0).",
    en: "Vietnamese input method for Windows, built on the OpenKey engine (GPL-3.0).",
  },
  // Update window and About tab
  "update.title": { vi: "Đã có bản EVKey mới", en: "A new version of EVKey is available" },
  "update.version": { vi: "Phiên bản {version}", en: "Version {version}" },
  "update.notes": { vi: "Có gì mới", en: "What's new" },
  "update.no_notes": { vi: "Bản này chưa có ghi chú phát hành.", en: "This release has no notes." },
  "update.install": { vi: "Cài đặt và mở lại", en: "Install and relaunch" },
  "update.downloading": { vi: "Đang tải…", en: "Downloading…" },
  "update.skip": { vi: "Bỏ qua bản này", en: "Skip this version" },
  "update.later": { vi: "Để sau", en: "Later" },
  "about.update.check": { vi: "Kiểm tra cập nhật", en: "Check for updates" },
  "hint.input_type.telex": { vi: "Telex: s, f, r, x, j cho dấu sắc, huyền, hỏi, ngã, nặng; aa, ee, oo, aw, ow, w cho â, ê, ô, ă, ơ, ư; dd cho đ. Ví dụ: as → á, dd → đ.", en: "Telex: s, f, r, x, j add the acute, grave, hook, tilde and dot marks; aa, ee, oo, aw, ow, w make â, ê, ô, ă, ơ, ư; dd makes đ. Example: as → á, dd → đ." },
  "hint.input_type.vni": { vi: "VNI: gõ số 1–5 cho dấu, 6–9 cho mũ, móc, trăng và đ. Ví dụ: a1 → á, d9 → đ.", en: "VNI: digits 1–5 add tone marks, 6–9 add hats, horns, breves and đ. Example: a1 → á, d9 → đ." },
  "hint.input_type.simple_telex1": { vi: "Simple Telex 1: như Telex nhưng w đứng một mình không thành ư, và [ ] không thành ơ, ư. Hợp khi hay gõ từ tiếng Anh có chữ w.", en: "Simple Telex 1: like Telex, but a lone w does not become ư and [ ] do not make ơ, ư. Good if you often type English words with w." },
  "hint.input_type.simple_telex2": { vi: "Simple Telex 2: như Telex nhưng [ ] không thành ơ, ư; w vẫn thành ư.", en: "Simple Telex 2: like Telex, but [ ] do not make ơ, ư; w still becomes ư." },
  "hint.tone_placement.new": { vi: "Kiểu mới: đặt dấu ở chữ thứ hai của oa, oe, uy. Ví dụ: hòa, thúy.", en: "New style: the mark goes on the second letter of oa, oe, uy. Example: hòa, thúy." },
  "hint.tone_placement.old": { vi: "Kiểu cũ: đặt dấu ở chữ thứ nhất của oa, oe, uy. Ví dụ: hoà, thuý.", en: "Old style: the mark goes on the first letter of oa, oe, uy. Example: hoà, thuý." },
  "hint.code_table.unicode": { vi: "Unicode: bảng mã chuẩn, đúng với hầu hết ứng dụng hiện nay.", en: "Unicode: the standard code table, right for almost every current app." },
  "hint.code_table.tcvn3": { vi: "TCVN3 (ABC): bảng mã cũ, chỉ dùng cho ứng dụng cần font .Vn như VnTime.", en: "TCVN3 (ABC): an old code table, only for apps that need .Vn fonts." },
  "hint.code_table.vni_windows": { vi: "VNI Windows: bảng mã cũ, chỉ dùng cho ứng dụng cần font VNI.", en: "VNI Windows: an old code table, only for apps that need VNI fonts." },
  "hint.code_table.unicode_compound": { vi: "Unicode tổ hợp: lưu dấu thành ký tự rời sau chữ cái. Dùng khi ứng dụng không hiển thị đúng Unicode dựng sẵn.", en: "Unicode compound: stores each mark as a separate character after the letter. Use when an app does not show precomposed Unicode correctly." },
  "hint.translate_action.show": { vi: "Chỉ hiện: bản dịch hiện trong cửa sổ nhỏ, không đụng vào văn bản của bạn.", en: "Show only: the translation appears in a small window and your text is left alone." },
  "hint.translate_action.copy": { vi: "Sao chép: hiện bản dịch và tự chép vào clipboard để bạn dán ở nơi khác.", en: "Copy: shows the translation and puts it on the clipboard for you to paste." },
  "hint.translate_action.replace": { vi: "Thay thế: dán bản dịch đè lên đoạn văn bản đang chọn. Clipboard cũ được khôi phục.", en: "Replace: pastes the translation over the selected text. Your previous clipboard is restored." },
  "hint.local-theme.system": { vi: "Theo Windows: tự đổi sáng hoặc tối theo cài đặt của Windows.", en: "Same as Windows: follows the Windows light or dark setting." },
  "hint.local-theme.light": { vi: "Sáng: luôn dùng giao diện sáng.", en: "Light: always use the light look." },
  "hint.local-theme.dark": { vi: "Tối: luôn dùng giao diện tối.", en: "Dark: always use the dark look." },
  "hint.local-glass-style.regular": { vi: "Thường: làm mờ nền phía sau, chữ dễ đọc hơn.", en: "Regular: blurs what is behind the window, easier to read." },
  "hint.local-glass-style.clear": { vi: "Trong suốt: lớp kính nhạt hơn, nhìn xuyên qua rõ hơn.", en: "Clear: a lighter glass layer that shows more of what is behind." },
  "about.update.checking": { vi: "Đang kiểm tra...", en: "Checking..." },
  "about.update.latest": { vi: "Bạn đang dùng bản mới nhất", en: "You are on the latest version" },
  "about.update.error": { vi: "Không kiểm tra được: {error}", en: "Could not check: {error}" },
  "about.update.title": { vi: "Tự động kiểm tra cập nhật", en: "Check for updates automatically" },
  "about.update.desc": { vi: "Kiểm tra khi mở EVKey và mỗi 24 giờ.", en: "Checks when EVKey starts and every 24 hours." },
  "about.github": { vi: "Mở trang GitHub", en: "Open GitHub page" },
  "about.credit": { vi: "Phát triển bởi ChrisNguyen", en: "Developed by ChrisNguyen" },

  // Popup menu
  "popup.vi_typing": { vi: "Gõ tiếng Việt", en: "Vietnamese typing" },
  "popup.input_type": { vi: "KIỂU GÕ", en: "INPUT METHOD" },
  "popup.table": { vi: "BẢNG MÃ", en: "CODE TABLE" },
  "popup.macro": { vi: "Quản lý gõ tắt…", en: "Manage macros…" },
  "popup.settings": { vi: "Mở cài đặt…", en: "Open settings…" },
  "popup.quit": { vi: "Thoát EVKey", en: "Quit EVKey" },

  // Basic / advanced split, welcome screen, backup, Gemini
  "common.advanced": { vi: "Nâng cao", en: "Advanced" },
  "welcome.title": { vi: "Chào mừng đến EVKey", en: "Welcome to EVKey" },
  "welcome.sub": { vi: "Ba bước nhỏ là gõ được tiếng Việt. Mọi thứ khác chỉnh sau ở các mục bên trái.", en: "Three small steps and you can type Vietnamese. Everything else can be tuned later from the left." },
  "welcome.step_input": { vi: "Chọn kiểu gõ bạn quen", en: "Pick the input method you know" },
  "welcome.step_toggle": { vi: "Chuyển Việt ⇄ Anh", en: "Switch Vietnamese ⇄ English" },
  "welcome.toggle_line": { vi: "Bấm {keys} bất cứ lúc nào. Đổi phím ở mục Phím tắt.", en: "Press {keys} at any time. Change it under Hotkeys." },
  "welcome.toggle_none": { vi: "Chưa đặt phím tắt; bấm icon EVKey ở khay hệ thống để chuyển.", en: "No hotkey set; click the EVKey tray icon to switch." },
  "welcome.step_try": { vi: "Gõ thử", en: "Try it" },
  "welcome.try.telex": { vi: "Ví dụ: vieejt nam → việt nam, ddaay → đây", en: "Example: vieejt nam → việt nam, ddaay → đây" },
  "welcome.try.vni": { vi: "Ví dụ: vie65t nam → việt nam, d9a6y → đây", en: "Example: vie65t nam → việt nam, d9a6y → đây" },
  "welcome.skip": { vi: "Bỏ qua", en: "Skip" },
  "welcome.start": { vi: "Bắt đầu", en: "Get started" },
  "macro.import.title": { vi: "Nhập từ UniKey / OpenKey", en: "Import from UniKey / OpenKey" },
  "macro.import.desc": { vi: "Chọn file gõ tắt đã xuất từ UniKey hoặc OpenKey.", en: "Pick a macro file exported from UniKey or OpenKey." },
  "macro.import.replace": { vi: "Thay toàn bộ", en: "Replace all" },
  "macro.import.button": { vi: "Chọn file…", en: "Choose file…" },
  "macro.import.done": { vi: "Đã nhập {n} từ gõ tắt", en: "Imported {n} macros" },
  "macro.import.confirm": { vi: "Xóa toàn bộ gõ tắt hiện có rồi nhập file này?", en: "Delete all current macros and import this file?" },
  "system.backup.heading": { vi: "Sao lưu", en: "Backup" },
  "system.export.title": { vi: "Xuất cấu hình", en: "Export settings" },
  "system.export.desc": { vi: "Lưu cài đặt và gõ tắt thành một file trong thư mục Downloads. Không chứa API key.", en: "Saves settings and macros as one file in Downloads. API keys are never included." },
  "system.export.button": { vi: "Xuất", en: "Export" },
  "system.export.done": { vi: "Đã lưu {path}", en: "Saved {path}" },
  "system.import.title": { vi: "Nhập cấu hình", en: "Import settings" },
  "system.import.desc": { vi: "Thay cài đặt và gõ tắt bằng file đã xuất.", en: "Replaces settings and macros with an exported file." },
  "system.import.button": { vi: "Chọn file…", en: "Choose file…" },
  "system.import.confirm": { vi: "Thay toàn bộ cài đặt và gõ tắt bằng file này?", en: "Replace all settings and macros with this file?" },
  "system.import.done": { vi: "Đã nhập cấu hình", en: "Settings imported" },
  "gemini.key.title": { vi: "API key Gemini", en: "Gemini API key" },
  "gemini.key.desc": { vi: "Cần cho Thêm dấu và Sửa/viết lại. Lấy key miễn phí ở aistudio.google.com. Key được giữ trong Windows Credential Manager, không ghi vào file cấu hình.", en: "Needed for Add diacritics and Fix/rewrite. Get a free key at aistudio.google.com. The key stays in Windows Credential Manager, never in the config files." },
  "gemini.key.save": { vi: "Kiểm tra & lưu", en: "Test & save" },
  "gemini.key.placeholder_set": { vi: "Đã lưu key (nhập key mới để thay)", en: "Key saved (type a new one to replace it)" },
  "gemini.key.placeholder_unset": { vi: "Dán API key vào đây", en: "Paste your API key here" },
  "gemini.key.status_set": { vi: "Đã có key.", en: "A key is saved." },
  "gemini.key.status_unset": { vi: "Chưa có key.", en: "No key saved yet." },
  "gemini.key.checking": { vi: "Đang kiểm tra…", en: "Checking…" },
  "gemini.key.saved": { vi: "Đã lưu API key", en: "API key saved" },
  "gemini.key.deleted": { vi: "Đã xóa API key", en: "API key deleted" },
};

let uiLang = "vi";

// Returns the text for `key` in the current UI language; `{name}` placeholders come from `params`.
function t(key, params = {}) {
  const entry = STRINGS[key];
  const text = entry ? entry[uiLang] ?? entry.vi : key;
  return text.replace(/\{(\w+)\}/g, (_, name) => params[name] ?? "");
}

// Anything other than "en" falls back to Vietnamese.
function setUiLanguage(lang) {
  uiLang = lang === "en" ? "en" : "vi";
}

// Fills every data-i18n element under `root` (and the document language).
function applyI18n(root = document) {
  document.documentElement.lang = uiLang;
  for (const el of root.querySelectorAll("[data-i18n]")) el.textContent = t(el.dataset.i18n);
  for (const el of root.querySelectorAll("[data-i18n-html]")) el.innerHTML = t(el.dataset.i18nHtml);
  for (const el of root.querySelectorAll("[data-i18n-title]")) el.title = t(el.dataset.i18nTitle);
  for (const el of root.querySelectorAll("[data-i18n-placeholder]")) el.placeholder = t(el.dataset.i18nPlaceholder);
  for (const el of root.querySelectorAll("[data-i18n-aria-label]")) el.setAttribute("aria-label", t(el.dataset.i18nAriaLabel));
}
