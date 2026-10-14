# Release notes

## v3.0.1

- Bảng hành động: kết quả từ menu Ctrl+Alt+A có nút **Thay thế** (dán đè lên đoạn đã bôi đen) thay cho nút sao chép. Phím tắt dịch vẫn giữ nút sao chép.
- Dịch bằng Google: lỗi mạng hoặc bị giới hạn (429) giờ hiện thông báo dễ hiểu, không còn hiện URL chứa văn bản đã bôi đen.
- Chế độ "dán đè" khi dịch ngoài ô nhập (trang web, PDF) chuyển sang sao chép, bản dịch nằm sẵn trong clipboard.
- Dịch thuật không còn là Beta.

## v3.0.0

- Bỏ hẳn bộ gõ TSF: EVKey chỉ còn cơ chế bắt phím Hook (thay đổi lớn, vì vậy lên 3.0). Cấu hình cũ có `backend = "tsf"` vẫn đọc được.
- Bảng hành động (Ctrl+Alt+A, chỉ mở khi đang ở ô nhập): Dịch, Thêm dấu, Sửa chính tả, Viết lại (trang trọng / ngắn gọn / thân thiện), Chuyển mã sang Unicode. Kết quả có nút sao chép. Giao diện liquid glass mới.
- Gemini: API key lưu trong Windows Credential Manager; dùng cho Thêm dấu, Sửa, Viết lại và có thể chọn làm dịch vụ dịch thay Google.
- Biến gõ tắt `{date}`, `{time}`, `{clipboard}`; nhập gõ tắt từ UniKey/OpenKey; xuất/nhập cấu hình.
- Cài đặt: màn chào mừng lần đầu, chia Cơ bản / Nâng cao, Kiểu gõ / Bảng mã / Dịch vụ dịch thành menu thả xuống, ô phụ âm thêm gọn hơn.
- Hiệu năng: CPU lúc rảnh giảm khoảng 3–4 lần (bỏ polling); độ trễ phím, RAM, khởi động giữ nguyên.
- Chưa kiểm chứng: bản macOS của các tính năng trên (chưa build được).

Gồm cả các thay đổi của 2.3.0 (chưa phát hành riêng):

### v2.3.0

- Ngôn ngữ giao diện: thêm English (Cài đặt, Popup, Banner Secure Input, thông báo lỗi). Chọn ở tab Hệ thống, đổi ngay không cần khởi động lại.
- Cập nhật trong app: tự kiểm tra khi mở và mỗi 24 giờ (tắt được), có nút "Kiểm tra cập nhật" ở tab Thông tin. Khi có bản mới hiện release notes với "Cài đặt & mở lại", "Bỏ qua bản này", "Để sau". Windows hiện UAC khi cài.
- Dịch thuật (Beta): bôi đen văn bản, bấm phím tắt (mặc định Ctrl+Alt+T, bấm vào ô rồi gõ phím bất kỳ để đổi) để dịch Việt ↔ Anh bằng Google Translate. Cửa sổ chỉ hiện bản dịch, có thanh thời gian chạy ngược phía trên, bấm vào để đóng. Cài đặt: việc làm thêm với bản dịch (chỉ hiện, sao chép, dán đè lên văn bản đã bôi đen) và thời gian hiển thị (ms hoặc giây, hoặc không tự đóng). Chỉ chạy với cơ chế bắt phím Hook.
- Tự viết hoa đầu câu: chọn riêng từng trường hợp (sau dấu chấm, dấu hỏi, dấu chấm than, khi xuống dòng). Dấu hỏi và dấu chấm than là mới.
- Sửa lỗi "Nhớ Việt/Anh theo ứng dụng" (Windows): đang ở chế độ Anh thì chuyển sang ứng dụng khác không được khôi phục chế độ của ứng dụng đó.
- Menu thả xuống kiểu mới trong cài đặt (thay hộp chọn mặc định của hệ thống).
- Viết lại chữ tiếng Việt và English trong cài đặt cho dễ hiểu hơn, kèm ví dụ; tab "Chính tả & gõ nhanh" tên tiếng Anh là "Smart typing" để không bị xuống dòng.
- Chưa kiểm chứng: bản macOS của các tính năng trên (chưa build được).

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
