import AppKit
import SwiftUI

/// Con trỏ bàn tay trên control bấm được. NSHostingView tự đặt lại mũi tên mỗi lần cập nhật con trỏ, nên push/pop
/// hay cursor rect đều bị đè; đặt lại bàn tay ở mỗi lần chuột di chuyển (onContinuousHover) thì luôn thắng.
private struct PointerCursor: ViewModifier {
    @State private var inside = false

    func body(content: Content) -> some View {
        content
            .onContinuousHover { phase in
                switch phase {
                case .active:
                    inside = true
                    NSCursor.pointingHand.set()
                case .ended:
                    inside = false
                    NSCursor.arrow.set()
                }
            }
            // Control biến mất ngay dưới chuột thì không có .ended: trả lại mũi tên ở đây.
            .onDisappear { if inside { NSCursor.arrow.set() } }
    }
}

extension View {
    func pointerCursor() -> some View { modifier(PointerCursor()) }
}
