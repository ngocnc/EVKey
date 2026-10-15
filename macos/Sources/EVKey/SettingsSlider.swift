import SwiftUI

/// Slider thương hiệu: track 5px, phần đã kéo là `brand-line`, núm trắng 20pt (hover ×1.1, kéo ×1.25).
struct SettingsSlider: View {
    @Binding var value: Int
    var range: ClosedRange<Int> = 0...100

    @Environment(\.colorScheme) private var scheme
    @ObservedObject private var appearance = GlassAppearance.shared
    @State private var dragging = false
    @State private var hovering = false

    var body: some View {
        GeometryReader { proxy in
            let knob: CGFloat = 20
            let span = max(proxy.size.width - knob, 1)
            let fraction = Double(value - range.lowerBound) / Double(range.upperBound - range.lowerBound)
            let x = CGFloat(fraction) * span
            ZStack(alignment: .leading) {
                Capsule().fill(Tokens(scheme).control).frame(height: 5)
                BrandLine().clipShape(Capsule()).frame(width: x + knob / 2, height: 5)
                Circle()
                    .fill(.white)
                    .shadow(color: dragging ? appearance.accent.shadow : .black.opacity(0.3), radius: dragging ? 5 : 2, y: 1)
                    .frame(width: knob, height: knob)
                    .scaleEffect(dragging ? 1.25 : hovering ? 1.1 : 1)
                    .offset(x: x)
            }
            .frame(maxHeight: .infinity)
            .contentShape(Rectangle())
            .onHover { hovering = $0 }
            .gesture(DragGesture(minimumDistance: 0)
                .onChanged { drag in
                    dragging = true
                    let f = min(max(Double((drag.location.x - knob / 2) / span), 0), 1)
                    value = range.lowerBound + Int((f * Double(range.upperBound - range.lowerBound)).rounded())
                }
                .onEnded { _ in dragging = false })
            .motion(.easeOut(duration: 0.12), value: dragging)
            .motion(.easeOut(duration: 0.12), value: hovering)
        }
        .frame(height: 24)
        .pointerCursor()
    }
}
