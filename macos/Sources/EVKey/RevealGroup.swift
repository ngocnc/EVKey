import SwiftUI

/// Vùng chứa hàng phụ thuộc: ẩn đến khi `isShown`, mở/đóng bằng chiều cao + độ mờ (320ms vào, 240ms ra).
/// Có thể ngắt giữa chừng; tắt hiệu ứng khi bật Reduce Motion.
struct RevealGroup<Content: View>: View {
    let isShown: Bool
    @ViewBuilder let content: () -> Content

    @Environment(\.accessibilityReduceMotion) private var reduceMotion

    var body: some View {
        VStack(spacing: 0) {
            if isShown {
                content().transition(.opacity)
            }
        }
        .frame(maxWidth: .infinity, alignment: .topLeading)
        .clipped()
        .animation(reduceMotion ? nil : isShown ? Motion.slide(0.32) : Motion.slide(0.24), value: isShown)
    }
}
