# Release notes

## v2.2.3

- Banner Secure Input: mờ dần cả cửa sổ (kể cả nền acrylic) thay vì hiệu ứng trượt gây giật; không áp lại acrylic mỗi lần hiện.
- Banner vừa khít nội dung (không còn cố định 360 px) và căn giữa theo vùng trang của trình duyệt, đúng cả khi dùng tab dọc.
- README cập nhật theo cơ chế hook mặc định và giao diện mới.
- Chưa kiểm chứng: vị trí banner trên trình duyệt thật.


## v2.2.2

- Banner Secure Input: hiệu ứng mờ dần khi hiện/ẩn.
- Cài đặt: thêm tuỳ chọn màu chủ đạo; trang Thông tin thiết kế lại; ba màu banner tách thành ba hàng, xem trước xuống cuối; nội dung mờ dần ở mép trên khi cuộn; bỏ thông báo "Đã lưu".
- Chế độ sáng: chữ trên nút/nhãn màu accent đậm hơn để dễ đọc.


## v2.2.1

- Banner Secure Input: góc cửa sổ được bo tròn bằng DWM, hết khung vuông xám quanh banner.
- Cài đặt: các chip phụ âm tùy chỉnh không còn dính nhau; mục Secure Input xếp lại cân đối (xem trước ở trên, slider cùng hàng với nhãn, ba màu trên một hàng).
- Chưa kiểm chứng: banner trên màn hình thật sau thay đổi này.

## v2.2.0

**Backend hook mặc định, giao diện liquid-glass mới.**

- Cơ chế gõ mặc định là hook bàn phím như UniKey (không cần thêm bàn phím Windows); TSF vẫn chọn được trong Hệ thống.
- Khay: bấm trái đổi V/E, bấm phải mở menu (kiểu gõ, bảng mã, gõ tắt, cài đặt). Popup không còn viền thừa và bóng.
- Cài đặt xếp lại giống bản macOS: icon cho từng tab, "Chế độ theo ứng dụng" và "Hoàn tác từ bằng Esc" chuyển sang tab Chính tả & gõ nhanh.
- Giao diện: chủ đề sáng/tối, kiểu kính (Thường/Trong suốt), độ đậm và màu phủ (một màu hoặc gradient), viền và slider rõ hơn ở chế độ sáng, bỏ hết shadow.
- Các mục phụ thuộc (ví dụ phụ âm tùy chỉnh) trượt vào/ra mượt, icon "+" căn giữa, icon ghim mới.
- Banner Secure Input hạ thấp hơn.
- Có thêm app macOS native (SwiftUI) dùng chung core.
- Chưa kiểm chứng: hook trong Chrome/VS Code; chế độ tối của giao diện mới.
## v2.1.0

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
