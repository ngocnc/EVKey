# EVKey

Bộ gõ tiếng Việt cá nhân cho Windows và macOS: nhận phím, trả về chữ tiếng Việt.

## Language

**Bộ gõ**:
Chương trình biến chuỗi phím thành chữ tiếng Việt, được hệ điều hành xem như một nguồn nhập.
_Avoid_: tool gõ, keyboard, IME (chỉ dùng khi nói về cơ chế của OS)

**Nguồn nhập**:
Một lựa chọn bàn phím/bộ gõ mà hệ điều hành cho người dùng chuyển qua lại (ví dụ "Tiếng Việt" và "Anh").
_Avoid_: chế độ, ngôn ngữ

**Kiểu gõ**:
Bảng quy ước phím nào sinh ra dấu hoặc chữ nào (Telex, VNI).
_Avoid_: input method, bộ gõ

**Âm tiết**:
Đơn vị chữ tiếng Việt gồm phụ âm đầu (tùy chọn), vần và dấu thanh. Bộ gõ xử lý từng âm tiết một.
_Avoid_: từ, word

**Dấu thanh**:
Một trong sáu thanh của âm tiết: ngang (không dấu), sắc, huyền, hỏi, ngã, nặng.
_Avoid_: dấu (chung chung)

**Dấu phụ**:
Dấu thay đổi chữ cái gốc: mũ (â ê ô), trăng (ă), móc (ơ ư) và gạch ngang (đ).
_Avoid_: dấu mũ (khi nói chung cả nhóm)

**Kiểu đặt dấu**:
Quy ước vị trí dấu thanh trên vần có hai nguyên âm: kiểu mới (`hòa`) hoặc kiểu cũ (`hoà`).

**Chuỗi phím**:
Các phím người dùng đã bấm để tạo ra một âm tiết, trước khi biến đổi.

**Chuỗi đang soạn**:
Âm tiết đang hiển thị (có gạch chân) nhưng chưa chốt vào văn bản.
_Avoid_: preedit, buffer

**Commit**:
Chốt chuỗi đang soạn thành văn bản thật.

**Khôi phục**:
Thay chuỗi đang soạn bằng chuỗi phím gốc, vì người dùng không định gõ tiếng Việt.

**Gõ nhanh**:
Quy ước một phím thay cho phụ âm đầu hoặc cuối đủ chữ (`cc` → `ch`, `f` → `ph`, `g` cuối → `ng`).

**Đặt dấu tự do**:
Cho phép gõ phím dấu sau khi âm tiết đã gõ xong, không bắt buộc ngay sau nguyên âm.

**Bảng mã đầu ra**:
Cách mã hóa chữ tiếng Việt khi chốt vào văn bản (Unicode, TCVN3, VNI Windows).
_Avoid_: font, chuyển mã

**Macro**:
Chuỗi gõ tắt được thay bằng đoạn văn bản dài hơn khi gặp ranh giới từ.
_Avoid_: gõ tắt, snippet

**Danh sách loại trừ**:
Tập ứng dụng mà bộ gõ không biến đổi phím, để phím đi thẳng vào ứng dụng.
_Avoid_: blocklist

**Tạm tắt**:
Bỏ qua biến đổi cho phần còn lại của âm tiết đang soạn; phím gõ tiếp theo vào nguyên văn cho tới khi hết âm tiết.
