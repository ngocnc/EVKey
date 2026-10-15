# EVKey

Bộ gõ tiếng Việt cho Windows 11 (x64) và macOS 13+ (Apple Silicon + Intel). Engine port từ [OpenKey](https://github.com/tuyenvm/OpenKey) nên repo là GPL-3.0.

## Chức năng

### Gõ tiếng Việt

- Kiểu gõ: Telex, VNI, Simple Telex 1, Simple Telex 2.
- Bảng mã: Unicode, TCVN3 (ABC), VNI Windows, Unicode tổ hợp.
- Đặt dấu kiểu mới (`hòa`) hoặc kiểu cũ (`hoà`), đặt dấu tự do.
- Kiểm tra chính tả, tự trả lại chữ gốc khi gõ sai chính tả.
- Tự khôi phục từ tiếng Anh (Beta): khoảng 770 từ thông dụng bị Telex biến đổi (`sorry`, `wrong`, `was`…) được trả lại như đã gõ; thêm từ riêng, loại trừ từ, Backspace để hoàn tác.
- Gõ nhanh: `cc`→`ch`, `gg`→`gi`, `nn`→`ng`…; phụ âm đầu (`f`→`ph`, `j`→`gi`, `w`→`qu`) và phụ âm cuối (`g`→`ng`, `h`→`nh`, `k`→`ch`).
- Tự viết hoa đầu câu (sau dấu chấm, dấu hỏi, dấu chấm than, khi xuống dòng).
- Phím tạm tắt tiếng Việt giữa từ (Ctrl hoặc Alt).

### Chuyển Việt / Anh

- Phím tắt chuyển chế độ tự đặt, kể cả tổ hợp chỉ có phím bổ trợ (ví dụ `ctrl+shift`).
- Nhớ Việt/Anh theo từng ứng dụng.
- Danh sách ứng dụng loại trừ: không gõ tiếng Việt trong các app đã chọn.

### Gõ tắt

- Bảng gõ tắt (`btw` → `by the way`), tự viết hoa theo từ gõ.
- Biến `{date}`, `{time}`, `{clipboard}`, `{cursor}` (đặt con trỏ sau khi bung).
- Nhập gõ tắt từ UniKey/OpenKey.

### Bảng hành động và dịch

- Bôi đen văn bản, bấm phím tắt để mở bảng hành động cạnh con trỏ:
  - Dịch Việt ↔ Anh (Google Translate hoặc Gemini).
  - Thêm dấu, Sửa chính tả, Viết lại (trang trọng / ngắn gọn / thân thiện) bằng Gemini.
  - Chuyển mã Unicode / TCVN3 / VNI; Bỏ dấu, HOA, thường, Hoa Đầu Từ (chạy tại máy).
- Phím tắt dịch nhanh, hiện bản dịch trong cửa sổ nhỏ; tuỳ chọn chỉ hiện, sao chép hoặc dán đè lên đoạn đã chọn.
- API key Gemini lưu trong Keychain (macOS) hoặc Credential Manager (Windows), không ghi vào file cấu hình.

### Giao diện

- Biểu tượng khay / menu bar với popup: bật tắt tiếng Việt, kiểu gõ, bảng mã, gõ tắt, cài đặt.
- Cửa sổ Cài đặt tự lưu, giao diện sáng/tối, kiểu kính Liquid Glass hoặc thường, nhiều màu chủ đạo.
- Ngôn ngữ giao diện tiếng Việt và English, đổi ngay không cần khởi động lại.
- Banner báo khi Secure Input (ô mật khẩu) đang chặn gõ tiếng Việt, nêu tên ứng dụng đang giữ.

### Hệ thống

- Khởi động cùng hệ thống.
- Windows: chạy với quyền quản trị để gõ được trong cửa sổ admin (qua Task Scheduler, không hỏi UAC mỗi lần).
- Tự kiểm tra và cài bản cập nhật trong app.
- Cấu hình trong `config.toml` và `macros.toml`, tự nạp lại khi file đổi; xuất/nhập cấu hình.
