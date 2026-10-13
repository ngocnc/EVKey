# Release notes

## v2.1.0 (chưa phát hành)

**Chuyển Việt/Anh thật sự, có phím tắt.** Trước đây V/E không phải một trạng thái của bộ gõ: `Win+Space` chỉ đổi bàn phím Windows và icon khay không biết gì. Từ bản này EVKey có chế độ riêng, dùng chung cho mọi ứng dụng.

- Phím tắt chuyển Việt/Anh, mặc định `alt+z`, đổi trong `config.toml` (`toggle_hotkey`) hoặc trong cửa sổ cài đặt. Hỗ trợ tổ hợp có phím (`alt+z`, `ctrl+space`, `win+f1`) và tổ hợp chỉ có phím bổ trợ (`ctrl+shift`, `alt+shift`); `none` để tắt. Đổi xong có hiệu lực trong khoảng 1 giây, không cần mở lại ứng dụng.
- Chế độ English: bộ gõ vẫn là bàn phím đang dùng nhưng không biến đổi gì; đổi chế độ giữa lúc đang gõ thì chữ đang soạn được chốt trước.
- Icon khay hiện V hoặc E theo chế độ thật (cập nhật trong chừng 0,3 giây), tooltip ghi phím tắt, bấm giữa chuột để đổi. Công tắc trong popup đổi chế độ thật.
- Mục "Bàn phím Windows" trong cài đặt: thêm/gỡ EVKey khỏi danh sách bàn phím (công tắc cũ của popup chuyển về đây).
- Lưu ý: phím Windows `Ctrl+Shift` / `Alt+Shift` vẫn đổi bàn phím riêng; nếu đặt `toggle_hotkey` trùng thì cả hai cùng chạy. Chọn tổ hợp khác hoặc tắt phím Windows trong Settings.
- Chưa kiểm chứng tự động: phím tắt trong ứng dụng thật (Notepad, trình duyệt...). Cần thử tay trước khi đánh tag.

## v2.0.0

**Ứng dụng riêng, có bộ cài và cài qua winget.**

- App khay (Tauri): popup cài nhanh (kiểu gõ, bảng mã, kiểm tra chính tả, đặt dấu tự do, gõ nhanh, tự viết hoa đầu câu, loại trừ app đang dùng) và cửa sổ cài đặt đầy đủ (mọi tuỳ chọn, trình soạn macro, danh sách loại trừ, khôi phục mặc định, mở thư mục cấu hình, khởi động cùng Windows).
- Bộ cài Inno Setup `EVKey-Setup-<version>.exe`: cài vào `C:\Program Files\EVKey\<version>\`, đăng ký bộ gõ, tuỳ chọn thêm vào danh sách bàn phím và tự khởi động, gỡ sạch (giữ `%APPDATA%\EVKey`). Hỗ trợ `/VERYSILENT`.
- Winget: `winget install chrisnguyen.EVKey` (sau khi gói được chấp nhận vào kho winget).
- CLI: `--hidden`, `--settings`, `--enable-keyboard`, `--disable-keyboard`, `--autostart on|off`, `--version`.
- Bộ cài chưa ký số: SmartScreen sẽ cảnh báo, chọn *More info → Run anyway*.
- Giới hạn: icon khay khi đó không đọc/chuyển được bàn phím đang dùng của ứng dụng khác (sửa ở v2.1.0).

## v1.0.0

**Bản đầu: bộ gõ TSF cho Windows 11 x64.**

- Engine port từ OpenKey sang Rust, so khớp từng sự kiện với engine C++ gốc trên 400.003 sự kiện.
- Telex/VNI, 4 bảng mã (Unicode, TCVN3, VNI Windows, Unicode tổ hợp), kiểu đặt dấu mới/cũ, đặt dấu tự do, gõ nhanh phụ âm, khôi phục từ sai chính tả, tự viết hoa đầu câu, macro, danh sách loại trừ, tạm tắt giữa từ, bỏ qua ô mật khẩu.
- Cấu hình bằng `config.toml` / `macros.toml` trong `%APPDATA%\EVKey`, tự nạp lại khi file đổi.
- Giới hạn: ứng dụng 32-bit không gõ được; `TextBox` WinForms cổ điển có thể mất chữ gõ sau lần commit đầu; chưa có macOS.
