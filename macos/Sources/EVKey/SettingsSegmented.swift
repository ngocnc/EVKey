import SwiftUI

/// Segmented control whose selection thumb slides between segments. The native control jumps when the value is
/// written back through the settings store, so this one animates the thumb itself.
struct SettingsSegmented<Value: Hashable>: View {
    let choices: [ChoiceOption<Value>]
    @Binding var selection: Value

    @Namespace private var thumb

    var body: some View {
        HStack(spacing: 0) {
            ForEach(choices) { choice in
                Text(choice.label)
                    .font(.system(size: 13, weight: selection == choice.value ? .semibold : .regular))
                    .frame(maxWidth: .infinity)
                    .padding(.vertical, 5)
                    .background {
                        if selection == choice.value {
                            RoundedRectangle(cornerRadius: 6, style: .continuous)
                                .fill(Color.primary.opacity(0.18))
                                .matchedGeometryEffect(id: "thumb", in: thumb)
                        }
                    }
                    .contentShape(Rectangle())
                    .pointerCursor()
                    .onTapGesture {
                        withAnimation(.easeInOut(duration: 0.2)) { selection = choice.value }
                    }
            }
        }
        .padding(2)
        .background(Color.primary.opacity(0.07), in: RoundedRectangle(cornerRadius: 8, style: .continuous))
        .animation(.easeInOut(duration: 0.2), value: selection)
    }
}
