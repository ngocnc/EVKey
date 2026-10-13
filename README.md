# EVKey

Bộ gõ tiếng Việt cá nhân (Rust). Windows 11 x64 dùng TSF thật; macOS (arm64) làm ở phase sau. Engine port từ [OpenKey](https://github.com/tuyenvm/OpenKey) nên repo là GPL-3.0 (xem `docs/adr/`).

## Cấu trúc

| Đường dẫn | Nội dung |
|---|---|
| `crates/engine` | Engine thuần (không phụ thuộc OS): port OpenKey, `Composer` cho IME, `settings` |
| `crates/tsf` | Text service Windows (`evkey_tsf.dll`) |
| `tools/difftest` | Harness C++ build engine OpenKey gốc để so sánh |
| `scripts/` | Cài/gỡ, kích hoạt, smoke test Notepad |

## Cài trên Windows

```powershell
cargo build -p evkey-tsf --release
powershell -ExecutionPolicy Bypass -File scripts/install.ps1   # tự xin quyền admin, 1 lần UAC
```

Chọn "EVKey" (English (United States)) bằng `Win+Space`. Gỡ: `scripts/uninstall.ps1`.
Log gỡ lỗi: tạo file `%LOCALAPPDATA%\EVKey\debug`, đọc `%LOCALAPPDATA%\EVKey\tsf.log`.

## Cấu hình

`%APPDATA%\EVKey\config.toml` và `macros.toml`, tự nạp lại khi file đổi (kiểm tra tối đa 1 lần/giây, không nạp giữa lúc đang soạn). File sai cú pháp thì giữ cấu hình cũ.

```toml
# config.toml — giá trị dưới đây là mặc định
input_type = "telex"            # telex | vni | simple_telex1 | simple_telex2
tone_placement = "new"          # new: hòa | old: hoà
code_table = "unicode"          # unicode | tcvn3 | vni_windows | unicode_compound
free_mark = false               # đặt dấu tự do
check_spelling = true
restore_if_wrong_spelling = true
quick_telex = false             # cc→ch gg→gi kk→kh nn→ng qq→qu pp→ph tt→th
quick_start_consonant = false   # f→ph j→gi w→qu
quick_end_consonant = false     # g→ng h→nh k→ch
upper_case_first_char = false
allow_consonant_zfwj = false
auto_caps_macro = false
temp_off_key = "none"           # none | ctrl | alt: bấm giữa từ để phần còn lại của từ không bị biến đổi
excluded_apps = []              # ví dụ ["game.exe"]
```

```toml
# macros.toml
btw = "by the way"
vn = "Việt Nam"
```

## Kiểm thử

```
cargo test -p evkey-engine --release     # parity với OpenKey C++ (400k sự kiện), composer, settings
scripts/smoke_notepad.ps1 -Type "Vieejt Nam, hoaf dduowcj." -Expected "Việt Nam, hòa được."
```

Tạo lại dữ liệu golden: `tools/difftest/gen_golden.bat` (cần MSVC Build Tools).
