import AppKit
import SwiftUI

/// Nền mesh 4 đốm của hero card (MASTER › Hero): trôi 18s, dừng khi cửa sổ/ứng dụng không focus, đứng yên khi Reduce Motion.
/// Dùng làm `.background { HeroMesh() }`; tự cắt theo khung của view cha qua `.clipShape` ở nơi gọi.
struct HeroMesh: View {
    @ObservedObject private var appearance = GlassAppearance.shared
    @Environment(\.accessibilityReduceMotion) private var reduceMotion
    @State private var active = true
    @State private var keyWindow = true

    /// (màu, độ đậm, tâm x, tâm y theo tỉ lệ khung, pha trôi). Bên trái rất nhạt, đậm dần sang phải để chữ luôn đọc rõ.
    private func blobs(_ accent: Accent) -> [(Color, Double, CGFloat, CGFloat, Double)] {
        [(accent.a, 0.18, 0.06, 0.30, 0), (accent.m, 0.30, 0.62, 1.15, 1.6),
         (accent.a, 0.42, 0.78, 0.05, 3.1), (accent.c, 0.55, 0.98, 0.80, 4.7)]
    }

    var body: some View {
        let running = active && keyWindow && !reduceMotion
        TimelineView(.animation(minimumInterval: 1 / 30, paused: !running)) { timeline in
            let t = running ? timeline.date.timeIntervalSinceReferenceDate : 0
            GeometryReader { proxy in
                let size = proxy.size
                ZStack {
                    // Nền là thẻ thường (do nơi gọi vẽ); chỉ các đốm màu nằm trên, nhạt bên trái để chữ dễ đọc.
                    ForEach(Array(blobs(appearance.accent).enumerated()), id: \.offset) { _, blob in
                        let angle = t / 18 * 2 * .pi + blob.4
                        Circle()
                            .fill(blob.0.opacity(blob.1))
                            .frame(width: size.width * 0.4, height: size.width * 0.4)
                            .position(
                                x: size.width * blob.2 + CGFloat(cos(angle)) * size.width * 0.06,
                                y: size.height * blob.3 + CGFloat(sin(angle)) * size.height * 0.15)
                            .blur(radius: 30)
                    }
                }
                .clipped()
            }
        }
        .onAppear { active = NSApp.isActive }
        .onReceive(NotificationCenter.default.publisher(for: NSApplication.didBecomeActiveNotification)) { _ in active = true }
        .onReceive(NotificationCenter.default.publisher(for: NSApplication.didResignActiveNotification)) { _ in active = false }
        .onReceive(NotificationCenter.default.publisher(for: NSWindow.didBecomeKeyNotification)) { _ in keyWindow = true }
        .onReceive(NotificationCenter.default.publisher(for: NSWindow.didResignKeyNotification)) { _ in keyWindow = false }
        .allowsHitTesting(false)
    }
}
