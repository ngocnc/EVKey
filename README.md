# EVKey

Bộ gõ tiếng Việt cá nhân (Rust). Windows 11 x64 dùng TSF thật; macOS (arm64) làm ở phase sau. Engine port từ [OpenKey](https://github.com/tuyenvm/OpenKey) nên repo là GPL-3.0.

## Cài đặt

### Cách 1: winget (khuyên dùng)

```powershell
winget install --id chrisnguyen.EVKey -e
```

Gói chỉ có trên winget sau khi manifest đầu tiên được Microsoft duyệt (xem `packaging/winget/`). Trước đó dùng cách 2.

### Cách 2: tải installer

Tải `EVKey-Setup-<version>.exe` từ [Releases](https://github.com/ngocnc/EVKey/releases), rồi chạy file. Installer cần quyền admin (UAC) để đăng ký bộ gõ.

**Cảnh báo SmartScreen:** installer chưa ký số nên Windows có thể hiện "Windows protected your PC". Bấm **More info** → **Run anyway**. Kiểm tra SHA256 trong phần ghi chú của release nếu muốn xác minh file.

Sau khi cài, chọn "EVKey" (English (United States)) bằng `Win+Space`. Installer tự thêm bộ gõ vào danh sách bàn phím và bật khởi động cùng Windows; có thể bỏ các tùy chọn này ở bước chọn tác vụ.

Cài im lặng (không hỏi gì, dùng mặc định):

```powershell
EVKey-Setup-2.0.0.exe /VERYSILENT /SUPPRESSMSGBOXES /NORESTART
```

Bỏ tùy chọn: thêm `/TASKS=""` (không thêm bàn phím, không khởi động cùng Windows).

### Gỡ cài đặt

Gỡ trong **Settings → Apps → Installed apps → EVKey**, hoặc `winget uninstall --id chrisnguyen.EVKey -e`. Gỡ sẽ đóng EVKey, bỏ khỏi danh sách bàn phím và khởi động cùng Windows, nhưng **giữ nguyên cấu hình** trong `%APPDATA%\EVKey`. Xóa thư mục đó nếu muốn bỏ hẳn cấu hình.

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

## Trạng thái (v2.0.0)

- Windows 11 x64: engine so khớp từng sự kiện với OpenKey gốc; bộ gõ, bộ cài, gỡ cài đã chạy thật trên máy tác giả.
- Icon khay không đọc được hay chuyển bàn phím đang dùng của app khác (đã đo: Windows không cho tiến trình ngoài làm việc này đáng tin cậy). Công tắc trong popup chỉ thêm/bớt EVKey khỏi danh sách bàn phím; chuyển Việt/Anh bằng `Win+Space`.
- Lỗi đã biết: `TextBox` WinForms (EDIT cổ điển) có thể mất chữ gõ sau lần commit đầu.
- Ứng dụng 32-bit không gõ được (chỉ build x64). macOS (arm64) chưa làm. Bộ cài chưa ký số.

## Phát triển

### Cấu trúc

| Đường dẫn | Nội dung |
|---|---|
| `crates/engine` | Engine thuần (không phụ thuộc OS): port OpenKey, `Composer` cho IME, `settings` |
| `crates/tsf` | Text service Windows (`evkey_tsf.dll`) |
| `tools/difftest` | Harness C++ build engine OpenKey gốc để so sánh |
| `installer/` | Script Inno Setup (`evkey.iss`) và ngôn ngữ tiếng Việt |
| `packaging/winget/` | Manifest winget |
| `scripts/` | Cài/gỡ dev, kích hoạt, smoke test Notepad, build installer |

### Build installer

```powershell
scripts/build-installer.ps1           # cargo build --release, rồi iscc → dist/EVKey-Setup-<version>.exe
scripts/build-installer.ps1 -NoBuild  # dùng binary có sẵn trong target/release
```

Cần Inno Setup 6 (`winget install --id JRSoftware.InnoSetup -e`).

### Cài dev từ mã nguồn

```powershell
cargo build -p evkey-tsf --release
powershell -ExecutionPolicy Bypass -File scripts/install.ps1   # tự xin quyền admin, 1 lần UAC
```

Chọn "EVKey" (English (United States)) bằng `Win+Space`. Gỡ: `scripts/uninstall.ps1`.
Log gỡ lỗi: tạo file `%LOCALAPPDATA%\EVKey\debug`, đọc `%LOCALAPPDATA%\EVKey\tsf.log`.

### Kiểm thử

```
cargo test -p evkey-engine --release     # parity với OpenKey C++ (400k sự kiện), composer, settings
scripts/smoke_notepad.ps1 -Type "Vieejt Nam, hoaf dduowcj." -Expected "Việt Nam, hòa được."
```

Tạo lại dữ liệu golden: `tools/difftest/gen_golden.bat` (cần MSVC Build Tools).

### Phát hành

Đẩy tag `v<version>` (khớp version trong `Cargo.toml`) để chạy `.github/workflows/release.yml`. Các bước một lần cần làm: xem comment đầu file workflow.
