# EVKey

Bộ gõ tiếng Việt cá nhân (Rust). Windows 11 x64 mặc định dùng hook bàn phím như UniKey (TSF vẫn chọn được); macOS (arm64 + x86_64) dùng CGEventTap. Engine port từ [OpenKey](https://github.com/tuyenvm/OpenKey) nên repo là GPL-3.0.

## Cài đặt

### Cách 1: winget (khuyên dùng)

```powershell
winget install --id chrisnguyen.EVKey -e
```

Gói chỉ có trên winget sau khi manifest đầu tiên được Microsoft duyệt (xem `packaging/winget/`). Trước đó dùng cách 2.

### Cách 2: tải installer

Tải `EVKey-Setup-<version>.exe` từ [Releases](https://github.com/ngocnc/EVKey/releases), rồi chạy file. Installer cần quyền admin (UAC) để đăng ký bộ gõ.

**Cảnh báo SmartScreen:** installer chưa ký số nên Windows có thể hiện "Windows protected your PC". Bấm **More info** → **Run anyway**. Kiểm tra SHA256 trong phần ghi chú của release nếu muốn xác minh file.

Sau khi cài, EVKey chạy ở khay hệ thống và gõ được ngay (cơ chế hook mặc định không cần thêm bàn phím Windows). Bấm trái icon khay để đổi Việt/Anh, bấm phải để mở menu (kiểu gõ, bảng mã, gõ tắt, cài đặt). Installer bật khởi động cùng Windows; có thể bỏ ở bước chọn tác vụ. Chọn `backend = "tsf"` nếu muốn dùng bàn phím TSF: khi đó chọn "EVKey" (English (United States)) bằng `Win+Space`.

Cài im lặng (không hỏi gì, dùng mặc định):

```powershell
EVKey-Setup-2.0.0.exe /VERYSILENT /SUPPRESSMSGBOXES /NORESTART
```

Bỏ tùy chọn: thêm `/TASKS=""` (không thêm bàn phím, không khởi động cùng Windows).

### macOS

Yêu cầu macOS 13 Ventura trở lên. Bản phát hành là universal (Apple Silicon + Intel) và **ad-hoc signed**, chưa notarize.

Cài bằng Homebrew:

```sh
brew tap ngocnc/evkey
brew install --cask evkey
```

Hoặc tải `EVKey-<version>-universal.dmg` từ [Releases](https://github.com/ngocnc/EVKey/releases), kéo `EVKey.app` vào `Applications`.

**Build từ source**: cần macOS 13+, Command Line Tools (Swift 6+), Rust stable. Chạy `scripts/build-macos.sh` ở root repo; script build lõi Rust (`evkey-core`) cho arm64 + x86_64, build app SwiftUI trong `macos/` (SwiftPM), đóng gói `build/EVKey.app` và `build/EVKey-<version>-universal.dmg` (ad-hoc signed, kèm SHA256 in ra terminal). Giao diện macOS là SwiftUI; phần engine dùng chung Rust qua C FFI.

**Lần mở đầu tiên**: vì app chưa notarize, Gatekeeper có thể chặn. Bản cài bằng Homebrew đã tự gỡ cờ quarantine. Bản tải dmg thì chuột phải `EVKey.app` → **Mở** → **Mở**.

**Cấp quyền**: vào **System Settings → Privacy & Security** và bật EVKey trong **Accessibility** và **Input Monitoring**. Chưa cấp quyền thì EVKey không gõ được; app tự kiểm tra lại và bắt đầu hoạt động khi quyền đã được cấp.

- Sau khi cập nhật phiên bản mới, bản ad-hoc có thể bị macOS reset quyền do chữ ký đổi. Khi build từ source, tạo một lần chứng chỉ tự ký tên `EVKey Local Signing` (Keychain Access → Certificate Assistant → Create a Certificate → Code Signing); `build-macos.sh` sẽ dùng nó nên quyền được giữ qua các lần build.
- Khi có **Secure Input** (ô nhập mật khẩu, hoặc app như Terminal bật "Secure Keyboard Entry"), EVKey tạm không xử lý tiếng Việt và hiện banner ở cạnh dưới màn hình, nêu tên ứng dụng đang giữ Secure Input. Windows có banner tương tự khi phát hiện ô mật khẩu.

Gỡ: `brew uninstall --cask evkey`. Thêm `--zap` để xóa cấu hình trong `~/Library/Application Support/EVKey` và LaunchAgent.

### Gỡ cài đặt (Windows)

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
toggle_hotkey = "alt+z"         # chuyển Việt/Anh: "none" để tắt; xem bên dưới
backend = "hook"                # "hook": hook phím như UniKey (mặc định, không cần thêm bàn phím); "tsf": bàn phím Windows
excluded_apps = []              # ví dụ ["game.exe"]
```

```toml
# macros.toml
btw = "by the way"
vn = "Việt Nam"
```

### Chuyển Việt / Anh

- Chế độ Việt/Anh dùng chung cho mọi ứng dụng. Mặc định là Việt. Chế độ Anh: phím đi thẳng tới ứng dụng, không bị biến đổi. (Với `backend = "tsf"` trạng thái này là một TSF global compartment.)
- `toggle_hotkey` là tổ hợp phím chuyển chế độ. Cú pháp (không phân biệt hoa thường, nối bằng `+`, không có khoảng trắng): không hoặc nhiều trong `ctrl|alt|shift|win`, kèm theo đúng một phím (`a`-`z`, `0`-`9`, `space`, `f1`-`f12`) hoặc không có phím nào. Ví dụ `alt+z`, `ctrl+shift`, `f8`. Tổ hợp không có phím (như `ctrl+shift`) kích hoạt khi nhả các phím bổ trợ mà không bấm phím nào khác ở giữa. `"none"` tắt phím tắt. Giá trị sai cú pháp làm việc nạp cấu hình báo lỗi.
- `backend = "hook"` (mặc định): EVKey nghe phím toàn cục và gõ lại bằng `SendInput`, không cần thêm bàn phím Windows và không cần DLL. Khi khởi động, EVKey tự gỡ mục bàn phím TSF (nếu có) để phím không bị xử lý hai lần. Hạn chế: cửa sổ chạy quyền admin chỉ gõ được nếu EVKey cũng chạy với quyền admin; Chrome/Electron có thể lệch dấu trong vài trường hợp (EVKey dùng cách chống autocomplete của OpenKey). `backend = "tsf"` giữ cơ chế cũ. Đổi `backend` cần khởi động lại EVKey; `evkey --mode` với hook chỉ có `vietnamese|english|toggle`.
- Đổi `toggle_hotkey` có hiệu lực trong khoảng 1 giây, không cần khởi động lại ứng dụng.
- **Lưu ý**: phím tắt đổi bố cục của Windows (Ctrl+Shift, Alt+Shift) vẫn hoạt động độc lập; EVKey không xử lý chúng. Nếu đặt `toggle_hotkey` trùng với tổ hợp này, cả hai sẽ cùng chạy.

## Trạng thái (v2.2.3)

- Windows 11 x64: engine so khớp từng sự kiện với OpenKey gốc; bộ gõ (hook), bộ cài, gỡ cài đã chạy thật trên máy tác giả.
- Khay: bấm trái đổi Việt/Anh, bấm phải mở menu. Cửa sổ Cài đặt (liquid-glass, tự lưu) có các tab: Gõ tiếng Việt, Chính tả & gõ nhanh, Gõ tắt, Phím tắt, Ứng dụng loại trừ, Giao diện (sáng/tối, kiểu kính, màu phủ, màu chủ đạo, banner Secure Input), Hệ thống, Thông tin. Mọi lựa chọn ghi vào `config.toml`.
- Banner Secure Input (Windows): hiện mờ dần, vừa khít nội dung, căn giữa theo vùng trang của trình duyệt (tính cả trường hợp tab dọc). Chưa kiểm chứng trên màn hình thật với mọi trình duyệt.
- Chưa kiểm chứng đủ: hook trong Chrome/VS Code; `TextBox` WinForms (EDIT cổ điển) có thể mất chữ sau lần commit đầu (chỉ gặp với TSF).
- Ứng dụng 32-bit không gõ được (chỉ build x64). Bộ cài chưa ký số. macOS: app SwiftUI chạy được (menu bar, Bảng điều khiển, Banner); gõ thật cần cấp quyền Accessibility và chưa có người dùng kiểm chứng trên nhiều ứng dụng.

## Phát triển

### Cấu trúc

| Đường dẫn | Nội dung |
|---|---|
| `crates/engine` | Engine thuần (không phụ thuộc OS): port OpenKey, `Composer` cho IME, `settings` |
| `crates/app` | App khay Windows (Tauri): hook bàn phím, cửa sổ Cài đặt, popup, banner (UI web ở `crates/app/ui`) |
| `crates/tsf` | Text service Windows (`evkey_tsf.dll`), chỉ dùng khi `backend = "tsf"` |
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
